pub mod connector;
pub use connector::{Connector, QueuePayload, QueuePayloadType};
use tracing::{debug, error};

use crate::resolver::in_process::model::feature_flag::FeatureFlag;
use crate::resolver::in_process::model::flag_parser::FlagParser;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug, Clone, PartialEq)]
pub enum StorageState {
    Ok,
    Stale,
    Error,
}

#[derive(Debug, Clone)]
pub struct StorageStateChange {
    pub storage_state: StorageState,
    pub changed_flags_keys: Vec<String>,
    pub sync_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct StorageQueryResult {
    pub feature_flag: Option<FeatureFlag>,
    pub flag_set_metadata: HashMap<String, serde_json::Value>,
}

pub struct FlagStore {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
    flag_set_metadata: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    state_sender: Sender<StorageStateChange>,
    connector: Arc<dyn Connector>,
}

impl FlagStore {
    pub fn new(connector: Arc<dyn Connector>) -> (Self, Receiver<StorageStateChange>) {
        let (state_sender, state_receiver) = channel(1000);

        (
            Self {
                flags: Arc::new(RwLock::new(HashMap::new())),
                flag_set_metadata: Arc::new(RwLock::new(HashMap::new())),
                state_sender,
                connector,
            },
            state_receiver,
        )
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        debug!("Initializing flag store");
        self.connector.init().await?;

        // Handle initial sync
        let stream = self.connector.get_stream();
        let mut receiver = stream.lock().await;
        debug!("Waiting for initial sync message");

        if let Some(receiver_ref) = receiver.as_mut() {
            match tokio::time::timeout(std::time::Duration::from_secs(5), receiver_ref.recv())
                .await?
            {
                Some(payload) => {
                    debug!("Received initial sync message");
                    match payload.payload_type {
                        QueuePayloadType::Data => {
                            debug!("Parsing flag data: {}", &payload.flag_data);
                            let parsing_result = FlagParser::parse_string(&payload.flag_data)?;
                            let mut flags_write = self.flags.write().await;
                            let mut metadata_write = self.flag_set_metadata.write().await;
                            *flags_write = parsing_result.flags;
                            *metadata_write = parsing_result.flag_set_metadata;
                            debug!("Successfully parsed {} flags", flags_write.len());
                        }
                        QueuePayloadType::Error => {
                            error!("Error in initial sync");
                            return Err(anyhow::anyhow!("Error in initial sync"));
                        }
                    }
                }
                None => {
                    error!("No initial sync message received");
                    return Err(anyhow::anyhow!("No initial sync message received"));
                }
            }
        }

        // Start continuous stream processing
        self.start_stream_listener().await;
        Ok(())
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        self.connector.shutdown().await
    }

    pub async fn get_flag(&self, key: &str) -> StorageQueryResult {
        let flags = self.flags.read().await;
        let metadata = self.flag_set_metadata.read().await;

        StorageQueryResult {
            feature_flag: flags.get(key).cloned(),
            flag_set_metadata: metadata.clone(),
        }
    }

    async fn start_stream_listener(&self) {
        let flags = self.flags.clone();
        let metadata = self.flag_set_metadata.clone();
        let sender = self.state_sender.clone();
        let stream = self.connector.get_stream();

        tokio::spawn(async move {
            let mut receiver = stream.lock().await;
            if let Some(receiver) = receiver.as_mut() {
                while let Some(payload) = receiver.recv().await {
                    match payload.payload_type {
                        QueuePayloadType::Data => {
                            if let Ok(parsing_result) = FlagParser::parse_string(&payload.flag_data)
                            {
                                let mut flags_write = flags.write().await;
                                let mut metadata_write = metadata.write().await;
                                *flags_write = parsing_result.flags;
                                *metadata_write = parsing_result.flag_set_metadata;
                                let _ = sender
                                    .send(StorageStateChange {
                                        storage_state: StorageState::Ok,
                                        changed_flags_keys: vec![],
                                        sync_metadata: payload.metadata.unwrap_or_default(),
                                    })
                                    .await;
                            }
                        }
                        QueuePayloadType::Error => {
                            let _ = sender
                                .send(StorageStateChange {
                                    storage_state: StorageState::Error,
                                    changed_flags_keys: vec![],
                                    sync_metadata: HashMap::new(),
                                })
                                .await;
                        }
                    }
                }
            }
        });
    }
}
