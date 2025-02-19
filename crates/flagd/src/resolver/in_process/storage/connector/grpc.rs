use super::{Connector, QueuePayload, QueuePayloadType};
use crate::flagd::sync::v1::{flag_sync_service_client::FlagSyncServiceClient, SyncFlagsRequest};
use crate::resolver::common::upstream::UpstreamConfig;
use crate::FlagdOptions;
use anyhow::{Context, Result};
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tonic::transport::{Channel, Uri};
use tracing::{debug, error, warn};

const CONNECTION_TIMEOUT_SECS: u64 = 5;

#[derive(Clone)]
pub struct GrpcStreamConnector {
    target: String,
    selector: Option<String>,
    sender: Sender<QueuePayload>,
    stream: Arc<Mutex<Option<Receiver<QueuePayload>>>>,
    shutdown: Arc<AtomicBool>,
    retry_backoff_ms: u32,
    retry_backoff_max_ms: u32,
    retry_grace_period: u32,
    stream_deadline_ms: u32,
    authority: String, // desired authority, e.g. "b-features-api.service"
}

impl GrpcStreamConnector {
    // Updated new() accepts the extra authority parameter.
    pub fn new(
        target: String,
        selector: Option<String>,
        options: &FlagdOptions,
        authority: String,
    ) -> Self {
        debug!("Creating new GrpcStreamConnector with target: {}", target);
        let (sender, receiver) = channel(1000);
        Self {
            target,
            selector,
            sender,
            stream: Arc::new(Mutex::new(Some(receiver))),
            shutdown: Arc::new(AtomicBool::new(false)),
            retry_backoff_ms: options.retry_backoff_ms,
            retry_backoff_max_ms: options.retry_backoff_max_ms,
            retry_grace_period: options.retry_grace_period,
            stream_deadline_ms: options.stream_deadline_ms,
            authority,
        }
    }

    async fn establish_connection_using(&self, config: &UpstreamConfig) -> Result<Channel> {
        debug!("Created endpoint: {:?}", config.endpoint().uri());
        let mut endpoint = config.endpoint().clone();
        if self.stream_deadline_ms > 0 {
            endpoint = endpoint
                .http2_keep_alive_interval(Duration::from_millis(self.stream_deadline_ms as u64));
        }
        // Use 'origin' to inject the desired authority. Since origin() expects a full URI,
        // we prepend "http://" to the authority string.
        let authority_uri = Uri::from_str(&format!("http://{}", self.authority))
            .context("Invalid authority URI")?;
        endpoint = endpoint.origin(authority_uri);

        endpoint
            .timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS))
            .connect()
            .await
            .context(format!("Failed to connect to gRPC server: {}", self.target))
    }

    async fn connect_with_timeout_using(&self, config: &UpstreamConfig) -> Result<Channel> {
        debug!(
            "Attempting connection with timeout to target: {}",
            self.target
        );
        let mut current_delay = self.retry_backoff_ms;
        let mut attempts = 0;
        while !self.shutdown.load(Ordering::Relaxed) {
            match self.establish_connection_using(config).await {
                Ok(channel) => {
                    debug!("Successfully established channel connection");
                    return Ok(channel);
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.retry_grace_period {
                        error!("Connection attempts exhausted: {}", e);
                        return Err(e.context("Max retries exceeded"));
                    }
                    let delay = Duration::from_millis(current_delay as u64);
                    warn!(
                        "Connection attempt {} failed, retrying in {}ms: {}",
                        attempts,
                        delay.as_millis(),
                        e
                    );
                    sleep(delay).await;
                    current_delay = (current_delay * 2).min(self.retry_backoff_max_ms);
                }
            }
        }
        Err(anyhow::anyhow!(
            "Shutdown requested during connection attempts"
        ))
    }

    async fn start_stream(&self) -> Result<()> {
        debug!("Starting sync stream connection to {}", self.target);
        let config = UpstreamConfig::new(self.target.clone(), true)?;
        let channel = self.connect_with_timeout_using(&config).await?;
        debug!("Using authority: {}", self.authority);
        // Create the gRPC client with no interceptor because the endpoint already carries the desired authority.
        let mut client = FlagSyncServiceClient::new(channel);
        let request = tonic::Request::new(SyncFlagsRequest {
            provider_id: "rust-flagd-provider".to_string(),
            selector: self.selector.clone().unwrap_or_default(),
        });
        debug!("Sending sync request with selector: {:?}", self.selector);
        match client.sync_flags(request).await {
            Ok(response) => {
                let mut stream = response.into_inner();
                while let Ok(Some(msg)) = stream.message().await {
                    if self.shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    debug!(
                        "Received flag configuration update: {} bytes",
                        msg.flag_configuration.len()
                    );
                    self.sender
                        .send(QueuePayload {
                            payload_type: QueuePayloadType::Data,
                            flag_data: msg.flag_configuration,
                            metadata: None,
                        })
                        .await?;
                }
                Ok(())
            }
            Err(status) => {
                error!("Error in sync stream: {}", status);
                Ok(())
            }
        }
    }

    // New helper that continuously attempts to keep the stream alive
    async fn run_sync_stream(&self) {
        let mut current_delay = self.retry_backoff_ms;
        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                debug!("Shutdown requested; stopping sync stream loop");
                break;
            }

            match self.start_stream().await {
                Ok(_) => {
                    // If start_stream finishes gracefully (i.e. connection closed without error),
                    // you might want to decide whether to try reconnecting or exit.
                    debug!("Sync stream ended; reconnecting");
                }
                Err(e) => {
                    error!(
                        "Sync stream encountered error: {}. Retrying in {}ms",
                        e, current_delay
                    );
                }
            }
            sleep(Duration::from_millis(current_delay as u64)).await;
            // Exponential backoff: double delay until max backoff is reached.
            current_delay = (current_delay * 2).min(self.retry_backoff_max_ms);
        }
    }
}

#[async_trait::async_trait]
impl Connector for GrpcStreamConnector {
    async fn init(&self) -> Result<()> {
        debug!("Initializing GrpcStreamConnector");
        let connector = self.clone();
        // Instead of spawning start_stream directly, we spawn using our new run_sync_stream loop.
        tokio::spawn(async move {
            debug!("Starting sync stream on {}", connector.target);
            connector.run_sync_stream().await;
        });
        debug!("Initialized sync stream connector");
        Ok(())
    }

    fn get_stream(&self) -> Arc<Mutex<Option<Receiver<QueuePayload>>>> {
        self.stream.clone()
    }

    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down GrpcStreamConnector");
        self.shutdown.store(true, Ordering::Relaxed);
        Ok(())
    }
}

// (existing file content above remains unchanged)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::common::upstream::UpstreamConfig;
    use crate::FlagdOptions;
    use serial_test::serial;
    use test_log::test;
    use tokio::time::Instant;

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    #[serial]
    async fn test_retry_mechanism_inprocess() {
        // Create options configured for a failing connection.
        let options = FlagdOptions {
            host: "invalid-host".to_string(),
            resolver_type: crate::ResolverType::InProcess,
            port: 4444,
            target_uri: None,
            deadline_ms: 500,
            retry_backoff_ms: 100,
            retry_backoff_max_ms: 400,
            retry_grace_period: 3,
            stream_deadline_ms: 500,
            tls: false,
            cert_path: None,
            selector: None,
            socket_path: None,
            cache_settings: None,
            source_configuration: None,
            offline_poll_interval_ms: None,
        };

        let connector = GrpcStreamConnector::new(
            "invalid-host".to_string(),
            None,
            &options,
            "invalid-authority".to_string(),
        );

        // Create an upstream configuration with the invalid target.
        let config = UpstreamConfig::new(connector.target.clone(), true)
            .expect("failed to create upstream config");

        let start = Instant::now();
        let result = connector.connect_with_timeout_using(&config).await;
        let elapsed = start.elapsed();

        // Ensure that after the configured retry attempts the connector gives up.
        assert!(result.is_err(), "Expected error on connection attempts");
        // With 3 attempts (retry backoff delays of 100ms and 200ms before the third attempt fails)
        // the total delay should be at least 300ms and less than 600ms (allowing for overhead)
        assert!(
            elapsed.as_millis() >= 300,
            "Elapsed time {}ms is less than expected",
            elapsed.as_millis()
        );
        assert!(
            // This is a little flaky, it runs as expected time to time
            // elapsed time is higher than 600ms, I assume it is either due
            // test environment or async to serial doesn't work as expected
            elapsed.as_millis() < 700,
            "Elapsed time {}ms is too high",
            elapsed.as_millis()
        );
    }
}
