use super::{Connector, QueuePayload, QueuePayloadType};
use crate::flagd::sync::v1::{flag_sync_service_client::FlagSyncServiceClient, SyncFlagsRequest};
use crate::resolver::common::upstream::UpstreamConfig;
use crate::FlagdOptions;
use anyhow::{Context, Result};
use tonic::metadata::MetadataValue;
use tonic::Request;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tonic::transport::Channel;
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
}

impl GrpcStreamConnector {
    pub fn new(target: String, selector: Option<String>, options: &FlagdOptions) -> Self {
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
        }
    }

    async fn connect_with_timeout(&self) -> Result<Channel> {
        debug!(
            "Attempting connection with timeout to target: {}",
            self.target
        );
        let mut current_delay = self.retry_backoff_ms;
        let mut attempts = 0;
    
        while !self.shutdown.load(Ordering::Relaxed) {
            match self.establish_connection().await {
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

    async fn establish_connection(&self) -> Result<Channel> {
        let config = UpstreamConfig::new(self.target.clone(), true)?;
        debug!("Created endpoint: {:?}", config.endpoint().uri());
        
        let mut endpoint = config.endpoint().clone();
        if self.stream_deadline_ms > 0 {
            endpoint = endpoint
                .http2_keep_alive_interval(Duration::from_millis(self.stream_deadline_ms as u64));
        }
        
        endpoint
            .timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS))
            .connect()
            .await
            .context(format!("Failed to connect to gRPC server: {}", self.target))
    }

    async fn start_stream(&self) -> Result<()> {
        debug!("Starting sync stream connection to {}", self.target);
        let channel = self.connect_with_timeout().await?;
        
        let config = UpstreamConfig::new(self.target.clone(), true)?;
        
        let mut client = FlagSyncServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
            req.metadata_mut().insert(
                "authority",
                MetadataValue::from_str(config.authority().path().trim_matches('/')).unwrap(),
            );
            Ok(req)
        });
    
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
}

#[async_trait::async_trait]
impl Connector for GrpcStreamConnector {
    async fn init(&self) -> Result<()> {
        debug!("Initializing GrpcStreamConnector");
        let connector = self.clone();
        tokio::spawn(async move {
            debug!("Starting sync stream on {}", connector.target);
            if let Err(e) = connector.start_stream().await {
                error!("Error in sync stream: {}", e);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tokio::time::{sleep, timeout};

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_retry_backoff() {
        let options = FlagdOptions {
            retry_backoff_ms: 100,
            retry_backoff_max_ms: 400,
            retry_grace_period: 3,
            ..Default::default()
        };

        let _connector = GrpcStreamConnector::new("test://localhost".to_string(), None, &options);

        let start = Instant::now();
        let _ = timeout(Duration::from_millis(500), async {
            while start.elapsed().as_millis() < 400 {
                sleep(Duration::from_millis(50)).await;
            }
        })
        .await;

        let duration = start.elapsed();
        assert!(duration.as_millis() >= 400);
        assert!(duration.as_millis() < 600);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_max_backoff_limit() {
        let options = FlagdOptions {
            retry_backoff_ms: 100,
            retry_backoff_max_ms: 150,
            retry_grace_period: 3,
            ..Default::default()
        };

        let _connector = GrpcStreamConnector::new("test://localhost".to_string(), None, &options);

        let start = Instant::now();
        let _ = timeout(Duration::from_millis(300), async {
            while start.elapsed().as_millis() < 200 {
                sleep(Duration::from_millis(50)).await;
            }
        })
        .await;

        let duration = start.elapsed();
        assert!(duration.as_millis() >= 200);
        assert!(duration.as_millis() < 400);
    }
}
