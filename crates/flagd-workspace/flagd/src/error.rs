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

impl From<flagd_evaluation_engine::error::FlagdEvaluationError> for FlagdError {
    fn from(error: flagd_evaluation_engine::error::FlagdEvaluationError) -> Self {
        match error {
            flagd_evaluation_engine::error::FlagdEvaluationError::Provider(s) => {
                FlagdError::Provider(s)
            }
            flagd_evaluation_engine::error::FlagdEvaluationError::Config(s) => {
                FlagdError::Config(s)
            }
            flagd_evaluation_engine::error::FlagdEvaluationError::Parse(s) => FlagdError::Parse(s),
            flagd_evaluation_engine::error::FlagdEvaluationError::Io(e) => FlagdError::Io(e),
            flagd_evaluation_engine::error::FlagdEvaluationError::Json(e) => FlagdError::Json(e),
        }
    }
}

impl From<open_feature_ofrep::OfrepError> for FlagdError {
    fn from(error: open_feature_ofrep::OfrepError) -> Self {
        match error {
            open_feature_ofrep::OfrepError::Provider(s) => FlagdError::Provider(s),
            open_feature_ofrep::OfrepError::Connection(s) => FlagdError::Connection(s),
            open_feature_ofrep::OfrepError::Config(s) => FlagdError::Config(s),
        }
    }
}

impl From<FlagdError> for flagd_evaluation_engine::error::FlagdEvaluationError {
    fn from(error: FlagdError) -> Self {
        match error {
            FlagdError::Provider(s) => {
                flagd_evaluation_engine::error::FlagdEvaluationError::Provider(s)
            }
            FlagdError::Config(s) => {
                flagd_evaluation_engine::error::FlagdEvaluationError::Config(s)
            }
            FlagdError::Parse(s) => flagd_evaluation_engine::error::FlagdEvaluationError::Parse(s),
            FlagdError::Io(e) => flagd_evaluation_engine::error::FlagdEvaluationError::Io(e),
            FlagdError::Json(e) => flagd_evaluation_engine::error::FlagdEvaluationError::Json(e),
            // Map other variants to Provider
            FlagdError::Connection(s) => {
                flagd_evaluation_engine::error::FlagdEvaluationError::Provider(s)
            }
            FlagdError::Sync(s) => {
                flagd_evaluation_engine::error::FlagdEvaluationError::Provider(s)
            }
            FlagdError::Channel(s) => {
                flagd_evaluation_engine::error::FlagdEvaluationError::Provider(s)
            }
            FlagdError::Timeout(s) => {
                flagd_evaluation_engine::error::FlagdEvaluationError::Provider(s)
            }
        }
    }
}
