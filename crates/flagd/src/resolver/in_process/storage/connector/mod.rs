pub mod file;
pub mod grpc;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct QueuePayload {
    pub payload_type: QueuePayloadType,
    pub flag_data: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, PartialEq)]
pub enum QueuePayloadType {
    Data,
    Error,
}

#[async_trait::async_trait]
pub trait Connector: Send + Sync {
    async fn init(&self) -> anyhow::Result<()>;
    async fn shutdown(&self) -> anyhow::Result<()>;
    fn get_stream(&self) -> Arc<Mutex<Option<Receiver<QueuePayload>>>>;
}
