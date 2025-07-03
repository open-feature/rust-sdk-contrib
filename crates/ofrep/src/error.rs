use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum OfrepError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Invalid configuration: {0}")]
    Config(String),
}

// Add implementations for error conversion
impl From<Box<dyn std::error::Error>> for OfrepError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        OfrepError::Provider(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for OfrepError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        OfrepError::Provider(error.to_string())
    }
}

impl From<anyhow::Error> for OfrepError {
    fn from(error: anyhow::Error) -> Self {
        OfrepError::Provider(error.to_string())
    }
}
