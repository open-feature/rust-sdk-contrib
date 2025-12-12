use thiserror::Error;

/// Error type for flagd operations
#[derive(Error, Debug)]
pub enum FlagdError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Invalid configuration: {0}")]
    Config(String),
    #[error("Sync error: {0}")]
    Sync(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Channel send error: {0}")]
    Channel(String),
}

impl From<Box<dyn std::error::Error>> for FlagdError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        FlagdError::Provider(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for FlagdError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        FlagdError::Provider(error.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for FlagdError {
    fn from(error: tokio::sync::mpsc::error::SendError<T>) -> Self {
        FlagdError::Channel(error.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for FlagdError {
    fn from(error: tokio::time::error::Elapsed) -> Self {
        FlagdError::Timeout(error.to_string())
    }
}
