use thiserror::Error;

/// Error message returned by Flagsmith SDK when a flag is not found.
///
/// This constant matches the hardcoded error message in the Flagsmith Rust SDK v2.0
/// (flagsmith/src/flagsmith/models.rs, Flags::get_flag method).
/// When a flag key doesn't exist in the flags HashMap and no default_flag_handler
/// is configured, the SDK returns a FlagsmithAPIError with this exact message.
///
/// Note: This is a known limitation of the current SDK error reporting. A more robust
/// approach would be for the SDK to provide a structured error variant (e.g.,
/// ErrorKind::FlagNotFound), but until that's available, we must rely on string matching.
/// This matching approach is used by other Flagsmith provider implementations as well.
const FLAGSMITH_FLAG_NOT_FOUND_MSG: &str = "API returned invalid response";

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

    /// Flag not found error
    #[error("Flag not found: {0}")]
    FlagNotFound(String),
}

/// Convert Flagsmith SDK errors to FlagsmithError
impl From<flagsmith::error::Error> for FlagsmithError {
    fn from(error: flagsmith::error::Error) -> Self {
        match error.kind {
            flagsmith::error::ErrorKind::FlagsmithAPIError => {
                // Check if this is a "flag not found" error by matching the SDK's error message
                if error.msg == FLAGSMITH_FLAG_NOT_FOUND_MSG {
                    FlagsmithError::FlagNotFound(error.msg)
                } else {
                    FlagsmithError::Api(error.msg)
                }
            }
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
            FlagsmithError::Evaluation(msg) => open_feature::EvaluationError {
                code: EvaluationErrorCode::General("Evaluation error".to_string()),
                message: Some(msg),
            },
            FlagsmithError::FlagNotFound(msg) => open_feature::EvaluationError {
                code: EvaluationErrorCode::FlagNotFound,
                message: Some(msg),
            },
        }
    }
}
