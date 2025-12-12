use open_feature_flagsmith::error::FlagsmithError;

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
    use flagsmith::flagsmith::models::Flags;

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

    // Verify the error message is what we expect
    assert_eq!(
        error.msg, "API returned invalid response",
        "SDK error message changed. Expected: 'API returned invalid response', got: '{}'",
        error.msg
    );

    // Convert to our error type and verify it's FlagNotFound
    let our_error: FlagsmithError = error.into();
    assert!(
        matches!(our_error, FlagsmithError::FlagNotFound(_)),
        "SDK flag not found error should convert to FlagsmithError::FlagNotFound"
    );
}

/// Test that our FlagsmithError conversion correctly identifies flag not found errors
#[test]
fn test_flagsmith_error_conversion_flag_not_found() {
    let sdk_error = flagsmith::error::Error::new(
        flagsmith::error::ErrorKind::FlagsmithAPIError,
        "API returned invalid response".to_string(),
    );

    let our_error: FlagsmithError = sdk_error.into();

    assert!(
        matches!(our_error, FlagsmithError::FlagNotFound(_)),
        "SDK flag not found error should convert to FlagsmithError::FlagNotFound"
    );
}

/// Test that other API errors are not mistaken for flag not found
#[test]
fn test_flagsmith_error_conversion_other_api_errors() {
    let sdk_error = flagsmith::error::Error::new(
        flagsmith::error::ErrorKind::FlagsmithAPIError,
        "Some other API error".to_string(),
    );

    let our_error: FlagsmithError = sdk_error.into();

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
