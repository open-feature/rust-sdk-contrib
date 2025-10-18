use super::{Connector, QueuePayload, QueuePayloadType};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio::time::Duration;
use tracing::{debug, error};

#[derive(Clone)]
pub struct FileConnector {
    flag_source_path: PathBuf,
    sender: Sender<QueuePayload>,
    stream: Arc<Mutex<Option<Receiver<QueuePayload>>>>,
    shutdown: Arc<AtomicBool>,
}

impl FileConnector {
    pub fn new(flag_source_path: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = channel(1);
        Self {
            flag_source_path: flag_source_path.into(),
            sender,
            stream: Arc::new(Mutex::new(Some(receiver))),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn watch_file(&self) -> Result<()> {
        let path = self.flag_source_path.clone();
        let sender = self.sender.clone();

        while !self.shutdown.load(Ordering::Relaxed) {
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    debug!("File change detected, sending update");
                    sender
                        .send(QueuePayload {
                            payload_type: QueuePayloadType::Data,
                            flag_data: content,
                            metadata: None,
                        })
                        .await?;
                }
                Err(e) => {
                    error!("File not found or inaccessible: {}", e);
                    sender
                        .send(QueuePayload {
                            payload_type: QueuePayloadType::Error,
                            flag_data: e.to_string(),
                            metadata: None,
                        })
                        .await?;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Connector for FileConnector {
    async fn init(&self) -> Result<()> {
        let connector = self.clone();

        // First read and send immediately
        let initial_content = tokio::fs::read_to_string(&self.flag_source_path).await?;
        self.sender
            .send(QueuePayload {
                payload_type: QueuePayloadType::Data,
                flag_data: initial_content,
                metadata: None,
            })
            .await?;

        // Then start the watch loop
        tokio::spawn(async move {
            if let Err(e) = connector.watch_file().await {
                error!("File watcher error: {}", e);
            }
        });

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn get_stream(&self) -> Arc<Mutex<Option<Receiver<QueuePayload>>>> {
        self.stream.clone()
    }
}
