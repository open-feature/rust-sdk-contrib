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

#[cfg(test)]
mod tests {
    use super::*;
    use flagsmith::flagsmith::models::Flags;

    /// This test validates that our FLAGSMITH_FLAG_NOT_FOUND_MSG constant matches
    /// the actual error message returned by the Flagsmith SDK when a flag is not found.
    ///
    /// If this test fails, it means the Flagsmith SDK has changed its error message,
    /// and we need to update our constant accordingly.
    ///
    /// This provides compile-time-like safety by ensuring that any SDK updates that
    /// change the error message will be caught during CI/CD testing.
    #[test]
    fn test_flag_not_found_error_message_matches_sdk() {
        // Create an empty Flags object with no default handler
        let flags = Flags::from_api_flags(&vec![], None, None).unwrap();

        // Try to get a non-existent flag - this should return the SDK's "flag not found" error
        let result = flags.get_flag("non_existent_flag");

        // Verify the error is returned
        assert!(result.is_err(), "Expected error for non-existent flag");

        let error = result.unwrap_err();

        // Verify the error kind is FlagsmithAPIError
        assert_eq!(
            error.kind,
            flagsmith::error::ErrorKind::FlagsmithAPIError,
            "Expected FlagsmithAPIError for flag not found"
        );

        // Verify the error message matches our constant
        assert_eq!(
            error.msg, FLAGSMITH_FLAG_NOT_FOUND_MSG,
            "FLAGSMITH_FLAG_NOT_FOUND_MSG constant does not match SDK error message. \
             SDK returned: '{}', but our constant is: '{}'. \
             Please update FLAGSMITH_FLAG_NOT_FOUND_MSG to match the SDK.",
            error.msg, FLAGSMITH_FLAG_NOT_FOUND_MSG
        );
    }

    /// Test that our FlagsmithError conversion correctly identifies flag not found errors
    #[test]
    fn test_flagsmith_error_conversion_flag_not_found() {
        // Create a flag not found error from the SDK
        let sdk_error = flagsmith::error::Error::new(
            flagsmith::error::ErrorKind::FlagsmithAPIError,
            FLAGSMITH_FLAG_NOT_FOUND_MSG.to_string(),
        );

        // Convert to our error type
        let our_error: FlagsmithError = sdk_error.into();

        // Verify it's converted to FlagNotFound variant
        assert!(
            matches!(our_error, FlagsmithError::FlagNotFound(_)),
            "SDK flag not found error should convert to FlagsmithError::FlagNotFound"
        );
    }

    /// Test that other API errors are not mistaken for flag not found
    #[test]
    fn test_flagsmith_error_conversion_other_api_errors() {
        // Create a different API error
        let sdk_error = flagsmith::error::Error::new(
            flagsmith::error::ErrorKind::FlagsmithAPIError,
            "Some other API error".to_string(),
        );

        // Convert to our error type
        let our_error: FlagsmithError = sdk_error.into();

        // Verify it's converted to Api variant, not FlagNotFound
        assert!(
            matches!(our_error, FlagsmithError::Api(_)),
            "Other API errors should convert to FlagsmithError::Api"
        );
    }

    /// Test OpenFeature error code mapping for flag not found
    #[test]
    fn test_open_feature_error_mapping_flag_not_found() {
        let our_error = FlagsmithError::FlagNotFound("test message".to_string());
        let of_error: open_feature::EvaluationError = our_error.into();

        assert_eq!(
            of_error.code,
            open_feature::EvaluationErrorCode::FlagNotFound,
            "FlagNotFound should map to EvaluationErrorCode::FlagNotFound"
        );
    }
}
