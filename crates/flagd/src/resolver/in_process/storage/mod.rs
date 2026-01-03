pub mod connector;
use crate::error::FlagdError;
pub use connector::{Connector, QueuePayload, QueuePayloadType};
use tracing::{debug, error, warn};

use crate::resolver::in_process::model::feature_flag::FeatureFlag;
use crate::resolver::in_process::model::flag_parser::FlagParser;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use tokio::sync::mpsc::{Receiver, Sender, channel};

/// State of the flag storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StorageState {
    /// Storage is healthy and up-to-date
    #[default]
    Ok,
    /// Storage data may be stale (connection issues)
    Stale,
    /// Storage encountered an error
    Error,
}

/// Represents a change in storage state with affected flags
#[derive(Debug, Clone, PartialEq)]
pub struct StorageStateChange {
    /// Current state of the storage
    pub storage_state: StorageState,
    /// Keys of flags that changed in this update
    pub changed_flags_keys: Vec<String>,
    /// Metadata from the sync operation
    pub sync_metadata: HashMap<String, serde_json::Value>,
}

impl Default for StorageStateChange {
    fn default() -> Self {
        Self {
            storage_state: StorageState::Ok,
            changed_flags_keys: Vec::new(),
            sync_metadata: HashMap::new(),
        }
    }
}

/// Result of querying a flag from storage
#[derive(Debug, Clone)]
pub struct StorageQueryResult {
    /// The feature flag if found
    pub feature_flag: Option<FeatureFlag>,
    /// Metadata associated with the flag set
    pub flag_set_metadata: HashMap<String, serde_json::Value>,
}

pub struct FlagStore {
    flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
    flag_set_metadata: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    state_sender: Sender<StorageStateChange>,
    connector: Arc<dyn Connector>,
    shutdown: Arc<AtomicBool>,
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
                shutdown: Arc::new(AtomicBool::new(false)),
            },
            state_receiver,
        )
    }

    pub async fn init(&self) -> Result<(), FlagdError> {
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
                            let flag_keys: Vec<String> =
                                parsing_result.flags.keys().cloned().collect();
                            *flags_write = parsing_result.flags;
                            *metadata_write = parsing_result.flag_set_metadata;
                            debug!("Successfully parsed {} flags", flags_write.len());

                            // Send initial state change so FileResolver knows init completed
                            let _ = self
                                .state_sender
                                .send(StorageStateChange {
                                    storage_state: StorageState::Ok,
                                    changed_flags_keys: flag_keys,
                                    sync_metadata: payload.metadata.unwrap_or_default(),
                                })
                                .await;
                        }
                        QueuePayloadType::Error => {
                            error!("Error in initial sync: {}", payload.flag_data);
                            return Err(FlagdError::Sync(format!(
                                "Error in initial sync: {}",
                                payload.flag_data
                            )));
                        }
                    }
                }
                None => {
                    error!("No initial sync message received");
                    return Err(FlagdError::Sync(
                        "No initial sync message received".to_string(),
                    ));
                }
            }
        }

        // Start continuous stream processing
        self.start_stream_listener().await;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), FlagdError> {
        debug!("Shutting down flag store");
        self.shutdown.store(true, Ordering::Relaxed);
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

    /// Compute which flags have changed between old and new flag sets
    fn compute_changed_flags(
        old_flags: &HashMap<String, FeatureFlag>,
        new_flags: &HashMap<String, FeatureFlag>,
    ) -> Vec<String> {
        let mut changed = Vec::new();

        // Check for modified or added flags
        for (key, new_flag) in new_flags {
            match old_flags.get(key) {
                Some(old_flag) if old_flag != new_flag => {
                    changed.push(key.clone());
                }
                None => {
                    changed.push(key.clone());
                }
                _ => {}
            }
        }

        // Check for deleted flags
        let old_keys: HashSet<_> = old_flags.keys().collect();
        let new_keys: HashSet<_> = new_flags.keys().collect();
        for key in old_keys.difference(&new_keys) {
            changed.push((*key).clone());
        }

        changed
    }

    async fn start_stream_listener(&self) {
        let flags = self.flags.clone();
        let metadata = self.flag_set_metadata.clone();
        let sender = self.state_sender.clone();
        let stream = self.connector.get_stream();
        let shutdown = self.shutdown.clone();

        tokio::spawn(async move {
            let mut receiver = stream.lock().await;
            if let Some(receiver) = receiver.as_mut() {
                while let Some(payload) = receiver.recv().await {
                    if shutdown.load(Ordering::Relaxed) {
                        debug!("Stream listener shutting down");
                        break;
                    }

                    match payload.payload_type {
                        QueuePayloadType::Data => {
                            match FlagParser::parse_string(&payload.flag_data) {
                                Ok(parsing_result) => {
                                    let mut flags_write = flags.write().await;
                                    let mut metadata_write = metadata.write().await;

                                    // Compute changed flags before updating
                                    let changed_keys = Self::compute_changed_flags(
                                        &flags_write,
                                        &parsing_result.flags,
                                    );

                                    let num_changes = changed_keys.len();
                                    *flags_write = parsing_result.flags;
                                    *metadata_write = parsing_result.flag_set_metadata;

                                    debug!(
                                        "Flag store updated: {} flags changed ({} total flags)",
                                        num_changes,
                                        flags_write.len()
                                    );

                                    let _ = sender
                                        .send(StorageStateChange {
                                            storage_state: StorageState::Ok,
                                            changed_flags_keys: changed_keys,
                                            sync_metadata: payload.metadata.unwrap_or_default(),
                                        })
                                        .await;
                                }
                                Err(e) => {
                                    warn!("Failed to parse flag data: {}", e);
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
                        QueuePayloadType::Error => {
                            error!("Received error from connector: {}", payload.flag_data);
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
            debug!("Stream listener stopped");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_flag(state: &str, default_variant: &str) -> FeatureFlag {
        FeatureFlag {
            state: state.to_string(),
            default_variant: default_variant.to_string(),
            variants: {
                let mut map = HashMap::new();
                map.insert("on".to_string(), json!(true));
                map.insert("off".to_string(), json!(false));
                map
            },
            targeting: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_compute_changed_flags_no_changes() {
        let mut flags = HashMap::new();
        flags.insert("flag1".to_string(), create_test_flag("ENABLED", "on"));
        flags.insert("flag2".to_string(), create_test_flag("ENABLED", "off"));

        let changed = FlagStore::compute_changed_flags(&flags, &flags);
        assert!(
            changed.is_empty(),
            "Expected no changes for identical flags"
        );
    }

    #[test]
    fn test_compute_changed_flags_added_flag() {
        let old_flags = HashMap::new();
        let mut new_flags = HashMap::new();
        new_flags.insert("flag1".to_string(), create_test_flag("ENABLED", "on"));

        let changed = FlagStore::compute_changed_flags(&old_flags, &new_flags);
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&"flag1".to_string()));
    }

    #[test]
    fn test_compute_changed_flags_removed_flag() {
        let mut old_flags = HashMap::new();
        old_flags.insert("flag1".to_string(), create_test_flag("ENABLED", "on"));
        let new_flags = HashMap::new();

        let changed = FlagStore::compute_changed_flags(&old_flags, &new_flags);
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&"flag1".to_string()));
    }

    #[test]
    fn test_compute_changed_flags_modified_flag() {
        let mut old_flags = HashMap::new();
        old_flags.insert("flag1".to_string(), create_test_flag("ENABLED", "on"));

        let mut new_flags = HashMap::new();
        new_flags.insert("flag1".to_string(), create_test_flag("ENABLED", "off")); // Changed default

        let changed = FlagStore::compute_changed_flags(&old_flags, &new_flags);
        assert_eq!(changed.len(), 1);
        assert!(changed.contains(&"flag1".to_string()));
    }

    #[test]
    fn test_compute_changed_flags_mixed_changes() {
        let mut old_flags = HashMap::new();
        old_flags.insert("flag1".to_string(), create_test_flag("ENABLED", "on"));
        old_flags.insert("flag2".to_string(), create_test_flag("ENABLED", "on"));
        old_flags.insert("flag3".to_string(), create_test_flag("ENABLED", "on"));

        let mut new_flags = HashMap::new();
        new_flags.insert("flag1".to_string(), create_test_flag("ENABLED", "on")); // Unchanged
        new_flags.insert("flag2".to_string(), create_test_flag("DISABLED", "on")); // Modified
        new_flags.insert("flag4".to_string(), create_test_flag("ENABLED", "on")); // Added
        // flag3 is removed

        let changed = FlagStore::compute_changed_flags(&old_flags, &new_flags);
        assert_eq!(changed.len(), 3);
        assert!(changed.contains(&"flag2".to_string())); // Modified
        assert!(changed.contains(&"flag3".to_string())); // Removed
        assert!(changed.contains(&"flag4".to_string())); // Added
        assert!(!changed.contains(&"flag1".to_string())); // Unchanged
    }

    #[test]
    fn test_storage_state_equality() {
        assert_eq!(StorageState::Ok, StorageState::Ok);
        assert_ne!(StorageState::Ok, StorageState::Error);
        assert_ne!(StorageState::Error, StorageState::Stale);
    }
}
