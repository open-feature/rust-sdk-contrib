use super::{Connector, QueuePayload, QueuePayloadType};
use crate::FlagdOptions;
use crate::error::FlagdError;
use crate::flagd::sync::v1::{SyncFlagsRequest, flag_sync_service_client::FlagSyncServiceClient};
use crate::resolver::common::upstream::UpstreamConfig;
use std::str::FromStr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender, channel};
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
    keep_alive_time_ms: u64,
    authority: Option<String>, // optional authority for custom name resolution (e.g. envoy://)
    provider_id: String,       // provider identifier for sync requests
    channel: Arc<Mutex<Option<Channel>>>, // reusable channel for connection pooling
    tls: bool,                 // whether to use TLS for connections
}

impl GrpcStreamConnector {
    /// Create a new GrpcStreamConnector for TCP connections
    pub fn new(
        target: String,
        selector: Option<String>,
        options: &FlagdOptions,
        authority: Option<String>,
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
            keep_alive_time_ms: options.keep_alive_time_ms,
            authority,
            provider_id: options
                .provider_id
                .clone()
                .unwrap_or_else(|| "rust-flagd-provider".to_string()),
            channel: Arc::new(Mutex::new(None)),
            tls: options.tls,
        }
    }

    /// Create a new GrpcStreamConnector for Unix socket connections
    pub fn new_unix(
        target: String,
        socket_path: String,
        selector: Option<String>,
        options: &FlagdOptions,
    ) -> Self {
        debug!(
            "Creating new GrpcStreamConnector for Unix socket: {}",
            socket_path
        );
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
            keep_alive_time_ms: options.keep_alive_time_ms,
            authority: None, // Unix sockets don't need custom authority
            provider_id: options
                .provider_id
                .clone()
                .unwrap_or_else(|| "rust-flagd-provider".to_string()),
            channel: Arc::new(Mutex::new(None)),
            tls: options.tls,
        }
    }

    async fn establish_connection_using(
        &self,
        config: &UpstreamConfig,
    ) -> Result<Channel, FlagdError> {
        debug!("Created endpoint: {:?}", config.endpoint().uri());
        let mut endpoint = config.endpoint().clone();

        // Configure connection and transport settings for optimal streaming
        endpoint = endpoint
            // HTTP/2 adaptive flow control - auto-adjusts window sizes based on RTT
            .http2_adaptive_window(true)
            // Explicit connect timeout (separate from request timeout)
            .connect_timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS))
            // TCP keepalive for OS-level dead connection detection
            .tcp_keepalive(Some(Duration::from_secs(60)));

        // Configure HTTP/2 keepalive for long-lived streaming connections
        // This keeps connections alive during idle periods and allows RPCs to start quickly
        if self.keep_alive_time_ms > 0 {
            endpoint = endpoint
                .http2_keep_alive_interval(Duration::from_millis(self.keep_alive_time_ms))
                .keep_alive_timeout(Duration::from_secs(20))
                .keep_alive_while_idle(true);
        }

        // Only set origin if authority is provided (for custom name resolution like envoy://)
        if let Some(ref authority) = self.authority {
            let authority_uri = Uri::from_str(&format!("http://{}", authority))
                .map_err(|e| FlagdError::Config(format!("Invalid authority URI: {}", e)))?;
            endpoint = endpoint.origin(authority_uri);
        }

        endpoint
            .timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS))
            .connect()
            .await
            .map_err(|e| {
                FlagdError::Connection(format!(
                    "Failed to connect to gRPC server {}: {}",
                    self.target, e
                ))
            })
    }

    async fn connect_with_timeout_using(
        &self,
        config: &UpstreamConfig,
    ) -> Result<Channel, FlagdError> {
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
                        return Err(FlagdError::Connection(format!(
                            "Max retries exceeded: {}",
                            e
                        )));
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
        Err(FlagdError::Connection(
            "Shutdown requested during connection attempts".to_string(),
        ))
    }

    /// Get or create a reusable channel connection
    async fn get_or_create_channel(&self) -> Result<Channel, FlagdError> {
        let mut channel_guard = self.channel.lock().await;
        if let Some(ref channel) = *channel_guard {
            debug!("Reusing existing channel connection");
            return Ok(channel.clone());
        }

        debug!("Creating new channel connection to {}", self.target);
        let config = UpstreamConfig::new(self.target.clone(), true, self.tls)?;
        let channel = self.connect_with_timeout_using(&config).await?;
        *channel_guard = Some(channel.clone());
        Ok(channel)
    }

    /// Invalidate the cached channel (e.g., after connection failure)
    async fn invalidate_channel(&self) {
        let mut channel_guard = self.channel.lock().await;
        *channel_guard = None;
        debug!("Invalidated cached channel");
    }

    async fn start_stream(&self) -> Result<(), FlagdError> {
        debug!("Starting sync stream connection to {}", self.target);
        let channel = self.get_or_create_channel().await?;
        debug!("Using authority: {:?}", self.authority);
        // Reuse channel for better performance - avoids connection overhead on reconnects
        let mut client = FlagSyncServiceClient::new(channel);
        let request = tonic::Request::new(SyncFlagsRequest {
            provider_id: self.provider_id.clone(),
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
                    // Stream ended gracefully - invalidate channel and reconnect
                    debug!("Sync stream ended; invalidating channel and reconnecting");
                    self.invalidate_channel().await;
                    current_delay = self.retry_backoff_ms; // Reset backoff on graceful close
                }
                Err(e) => {
                    // Error occurred - invalidate channel for fresh connection on retry
                    error!(
                        "Sync stream encountered error: {}. Retrying in {}ms",
                        e, current_delay
                    );
                    self.invalidate_channel().await;
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
    async fn init(&self) -> Result<(), FlagdError> {
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

    async fn shutdown(&self) -> Result<(), FlagdError> {
        debug!("Shutting down GrpcStreamConnector");
        self.shutdown.store(true, Ordering::Relaxed);
        Ok(())
    }
}

// (existing file content above remains unchanged)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlagdOptions;
    use crate::resolver::common::upstream::UpstreamConfig;
    use serial_test::serial;
    use test_log::test;
    use tokio::net::TcpListener;
    use tokio::time::Instant;

    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    #[serial]
    async fn test_retry_mechanism_inprocess() {
        // Bind to a port but don't accept connections - this causes immediate connection failures
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        // Drop the listener immediately to ensure the port rejects connections
        drop(listener);

        // Create options configured for a failing connection.
        let mut options = FlagdOptions::default();
        options.host = addr.ip().to_string();
        options.resolver_type = crate::ResolverType::InProcess;
        options.port = addr.port();
        options.deadline_ms = 100; // Short timeout for fast failures
        options.retry_backoff_ms = 100;
        options.retry_backoff_max_ms = 400;
        options.retry_grace_period = 3;
        options.stream_deadline_ms = 500;
        options.tls = false;
        options.cache_settings = None;

        let target = format!("{}:{}", addr.ip(), addr.port());
        let connector = GrpcStreamConnector::new(target.clone(), None, &options, None);

        // Create an upstream configuration with the invalid target.
        let config =
            UpstreamConfig::new(target, false, false).expect("failed to create upstream config");

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
            elapsed.as_millis() < 600,
            "Elapsed time {}ms is too high",
            elapsed.as_millis()
        );
    }
}
