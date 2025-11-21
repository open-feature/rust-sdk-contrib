use thiserror::Error;

/// Known error messages from Flagsmith SDK
const FLAGSMITH_FLAG_NOT_FOUND: &str = "API returned invalid response";

/// Custom error types for the Flagsmith provider.
#[derive(Error, Debug, PartialEq)]
pub enum FlagsmithError {
    /// Configuration error (invalid options during initialization)
    #[error("Configuration error: {0}")]
    Config(String),

    /// API or network error (connection issues, timeouts, etc.)
    #[error("API error: {0}")]
    Api(String),

    /// Flag evaluation error (flag not found, type mismatch, etc.)
    #[error("Evaluation error: {0}")]
    Evaluation(String),
}

/// Convert Flagsmith SDK errors to FlagsmithError
impl From<flagsmith::error::Error> for FlagsmithError {
    fn from(error: flagsmith::error::Error) -> Self {
        match error.kind {
            flagsmith::error::ErrorKind::FlagsmithAPIError => FlagsmithError::Api(error.msg),
            flagsmith::error::ErrorKind::FlagsmithClientError => {
                FlagsmithError::Evaluation(error.msg)
            }
        }
    }
}

/// Convert URL parse errors to FlagsmithError
impl From<url::ParseError> for FlagsmithError {
    fn from(error: url::ParseError) -> Self {
        FlagsmithError::Config(format!("Invalid URL: {}", error))
    }
}

/// Convert serde_json errors to FlagsmithError
impl From<serde_json::Error> for FlagsmithError {
    fn from(error: serde_json::Error) -> Self {
        FlagsmithError::Evaluation(format!("JSON parse error: {}", error))
    }
}

/// Map FlagsmithError to OpenFeature EvaluationError
impl From<FlagsmithError> for open_feature::EvaluationError {
    fn from(error: FlagsmithError) -> Self {
        use open_feature::EvaluationErrorCode;

        match error {
            FlagsmithError::Config(msg) => open_feature::EvaluationError {
                code: EvaluationErrorCode::General("Configuration error".to_string()),
                message: Some(msg),
            },
            FlagsmithError::Api(msg) => open_feature::EvaluationError {
                code: EvaluationErrorCode::ProviderNotReady,
                message: Some(msg),
            },
            FlagsmithError::Evaluation(msg) => {
                if msg == FLAGSMITH_FLAG_NOT_FOUND {
                    open_feature::EvaluationError {
                        code: EvaluationErrorCode::FlagNotFound,
                        message: Some(msg),
                    }
                } else {
                    open_feature::EvaluationError {
                        code: EvaluationErrorCode::General("Evaluation error".to_string()),
                        message: Some(msg),
                    }
                }
            }
        }
    }
}
