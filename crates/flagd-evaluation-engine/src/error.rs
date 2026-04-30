use thiserror::Error;

/// Error type for flagd-evaluation operations
#[derive(Error, Debug)]
pub enum FlagdEvaluationError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Invalid configuration: {0}")]
    Config(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<Box<dyn std::error::Error>> for FlagdEvaluationError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        FlagdEvaluationError::Provider(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for FlagdEvaluationError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        FlagdEvaluationError::Provider(error.to_string())
    }
}
