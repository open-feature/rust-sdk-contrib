pub mod file;
pub mod grpc;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;

/// Payload sent through the connector stream containing flag data or errors
#[derive(Debug, Clone)]
pub struct QueuePayload {
    /// Type of payload (Data or Error)
    pub payload_type: QueuePayloadType,
    /// Flag configuration data (JSON string) or error message
    pub flag_data: String,
    /// Optional metadata associated with the sync
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Type of payload in the queue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueuePayloadType {
    /// Successful data payload
    Data,
    /// Error payload
    Error,
}

use crate::error::FlagdError;

/// Trait for flag configuration connectors (gRPC, file, etc.)
///
/// Connectors are responsible for fetching flag configurations from external sources
/// and providing them as a stream of payloads. Implementations must be thread-safe
/// (Send + Sync) as they may be used in async contexts.
#[async_trait::async_trait]
pub trait Connector: Send + Sync {
    /// Initialize the connector and start fetching data
    async fn init(&self) -> Result<(), FlagdError>;

    /// Gracefully shutdown the connector and release resources
    async fn shutdown(&self) -> Result<(), FlagdError>;

    /// Get the stream of payloads from this connector
    fn get_stream(&self) -> Arc<Mutex<Option<Receiver<QueuePayload>>>>;
}
