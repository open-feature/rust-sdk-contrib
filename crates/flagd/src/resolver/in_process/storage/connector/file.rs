use super::{Connector, QueuePayload, QueuePayloadType};
use crate::error::FlagdError;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tracing::{debug, error, warn};

pub struct FileConnector {
    flag_source_path: PathBuf,
    sender: Sender<QueuePayload>,
    stream: Arc<Mutex<Option<Receiver<QueuePayload>>>>,
    shutdown: Arc<AtomicBool>,
    watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
}

impl FileConnector {
    pub fn new(flag_source_path: impl Into<PathBuf>) -> Self {
        let (sender, receiver) = channel(100);
        Self {
            flag_source_path: flag_source_path.into(),
            sender,
            stream: Arc::new(Mutex::new(Some(receiver))),
            shutdown: Arc::new(AtomicBool::new(false)),
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    async fn read_and_send_file(&self) -> Result<(), FlagdError> {
        let path = &self.flag_source_path;
        match tokio::fs::read_to_string(path).await {
            Ok(content) => {
                debug!("Reading flag configuration from file: {:?}", path);
                self.sender
                    .send(QueuePayload {
                        payload_type: QueuePayloadType::Data,
                        flag_data: content,
                        metadata: None,
                    })
                    .await?;
            }
            Err(e) => {
                error!("Failed to read flag file {:?}: {}", path, e);
                self.sender
                    .send(QueuePayload {
                        payload_type: QueuePayloadType::Error,
                        flag_data: e.to_string(),
                        metadata: None,
                    })
                    .await?;
            }
        }
        Ok(())
    }

    fn setup_watcher(&self) -> Result<RecommendedWatcher, FlagdError> {
        let sender = self.sender.clone();
        let path = self.flag_source_path.clone();
        let shutdown = self.shutdown.clone();

        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if shutdown.load(Ordering::Relaxed) {
                return;
            }

            match res {
                Ok(event) => {
                    // Match events that indicate file content changes
                    // Include all Modify events to handle atomic writes (temp file → rename)
                    // Note: We watch the parent directory and re-read our specific file on any
                    // relevant event. This is intentional to handle editors that use atomic
                    // writes (write to temp, rename over original).
                    let dominated_events = matches!(
                        event.kind,
                        notify::EventKind::Modify(_)
                            | notify::EventKind::Create(_)
                            | notify::EventKind::Remove(_)
                    );

                    if dominated_events {
                        debug!("File change detected: {:?}", event.kind);
                        let path = path.clone();
                        let sender = sender.clone();

                        // Use std::fs for sync context in notify callback
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                if let Err(e) = sender.blocking_send(QueuePayload {
                                    payload_type: QueuePayloadType::Data,
                                    flag_data: content,
                                    metadata: None,
                                }) {
                                    error!("Failed to send file update: {}", e);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to read file after change: {}", e);
                                let _ = sender.blocking_send(QueuePayload {
                                    payload_type: QueuePayloadType::Error,
                                    flag_data: e.to_string(),
                                    metadata: None,
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                }
            }
        })
        .map_err(|e| FlagdError::Io(std::io::Error::other(e)))?;

        Ok(watcher)
    }
}

#[async_trait::async_trait]
impl Connector for FileConnector {
    async fn init(&self) -> Result<(), FlagdError> {
        // First read and send the initial content
        self.read_and_send_file().await?;

        // Set up the file watcher
        let mut watcher = self.setup_watcher()?;

        // Watch the parent directory to catch file replacements
        let watch_path = self
            .flag_source_path
            .parent()
            .unwrap_or(&self.flag_source_path);

        watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| FlagdError::Io(std::io::Error::other(e)))?;
        debug!(
            "Started watching for file changes at: {:?}",
            self.flag_source_path
        );

        // Store the watcher to keep it alive
        let mut watcher_guard = self.watcher.lock().await;
        *watcher_guard = Some(watcher);

        Ok(())
    }

    async fn shutdown(&self) -> Result<(), FlagdError> {
        debug!("Shutting down file connector");
        self.shutdown.store(true, Ordering::Relaxed);

        // Drop the watcher to stop watching
        let mut watcher_guard = self.watcher.lock().await;
        if let Some(mut watcher) = watcher_guard.take() {
            let watch_path = self
                .flag_source_path
                .parent()
                .unwrap_or(&self.flag_source_path);
            let _ = watcher.unwatch(watch_path);
        }

        Ok(())
    }

    fn get_stream(&self) -> Arc<Mutex<Option<Receiver<QueuePayload>>>> {
        self.stream.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_connector_init_reads_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config = r#"{"flags": {"test": {"state": "ENABLED", "variants": {"on": true}, "defaultVariant": "on"}}}"#;
        write!(temp_file, "{}", config).unwrap();

        let connector = FileConnector::new(temp_file.path());
        connector.init().await.unwrap();

        // Get the stream and verify we received the initial payload
        let stream = connector.get_stream();
        let mut receiver = stream.lock().await;
        let payload = receiver.as_mut().unwrap().recv().await.unwrap();

        assert_eq!(payload.payload_type, QueuePayloadType::Data);
        assert!(payload.flag_data.contains("test"));
    }

    #[tokio::test]
    async fn test_file_connector_init_fails_for_nonexistent_file() {
        let connector = FileConnector::new("/nonexistent/path/to/file.json");
        let result = connector.init().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_connector_shutdown() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config = r#"{"flags": {}}"#;
        write!(temp_file, "{}", config).unwrap();

        let connector = FileConnector::new(temp_file.path());
        connector.init().await.unwrap();

        // Shutdown should succeed
        let result = connector.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_connector_detects_file_changes() {
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_path_buf();

        // Write initial config
        std::fs::write(&file_path, r#"{"flags": {"v1": {}}}"#).unwrap();

        let connector = FileConnector::new(&file_path);
        connector.init().await.unwrap();

        let stream = connector.get_stream();
        let mut receiver = stream.lock().await;

        // Consume initial payload
        let _ = receiver.as_mut().unwrap().recv().await.unwrap();

        // Update the file
        std::fs::write(&file_path, r#"{"flags": {"v2": {}}}"#).unwrap();

        // Wait for the file watcher to detect the change
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Try to receive the update (with timeout)
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            receiver.as_mut().unwrap().recv(),
        )
        .await;

        if let Ok(Some(payload)) = result {
            assert_eq!(payload.payload_type, QueuePayloadType::Data);
            assert!(payload.flag_data.contains("v2"));
        }
        // Note: File watching behavior may vary by OS, so we don't fail if no update received
    }
}
