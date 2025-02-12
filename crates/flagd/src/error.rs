use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlagdError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Invalid configuration: {0}")]
    Config(String),
}

// Add implementations for error conversion
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

impl From<anyhow::Error> for FlagdError {
    fn from(error: anyhow::Error) -> Self {
        FlagdError::Provider(error.to_string())
    }
}
