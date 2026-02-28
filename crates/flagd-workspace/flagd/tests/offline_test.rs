use std::fs::File;
use std::time::Duration;

use common::FLAGD_CONFIG;
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, Value};
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use std::io::Write;
use tempfile::NamedTempFile;
use test_log::test;

mod common;

#[test(tokio::test)]
async fn test_in_process_file_sync() {
    // Create temporary config file
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", FLAGD_CONFIG).unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    let provider = FlagdProvider::new(FlagdOptions {
        source_configuration: Some(file_path.clone()),
        resolver_type: ResolverType::File,
        cache_settings: None,
        ..Default::default()
    })
    .await
    .unwrap();

    let context = EvaluationContext::default();

    // Test initial state
    let bool_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(bool_result.value, true);

    // Test file update
    let updated_config =
        FLAGD_CONFIG.replace("\"defaultVariant\": \"on\"", "\"defaultVariant\": \"off\"");
    let mut file = File::create(&file_path).unwrap();
    write!(file, "{}", updated_config).unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;

    let updated_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(updated_result.value, false);
}

#[test(tokio::test)]
async fn test_file_connector_error_handling() {
    // Test with non-existent file
    let provider = FlagdProvider::new(FlagdOptions {
        source_configuration: Some("/nonexistent/path".to_string()),
        resolver_type: ResolverType::File,
        ..Default::default()
    })
    .await;

    assert!(provider.is_err());

    // Test with invalid JSON
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "invalid json").unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        source_configuration: Some(temp_file.path().to_str().unwrap().to_string()),
        resolver_type: ResolverType::File,
        ..Default::default()
    })
    .await;

    assert!(provider.is_err());
}

#[test(tokio::test)]
async fn test_file_connector_file_deletion() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", FLAGD_CONFIG).unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    let provider = FlagdProvider::new(FlagdOptions {
        source_configuration: Some(file_path.clone()),
        resolver_type: ResolverType::File,
        cache_settings: None,
        ..Default::default()
    })
    .await
    .unwrap();

    // Verify initial flag state
    let context = EvaluationContext::default();
    let initial_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(initial_result.value, true);

    // Delete file and wait for error to be logged
    // Error is visible if tracing_subscriber::fmt::init() is called before the provider is created
    drop(temp_file);
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify provider still returns last known values
    let cached_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(cached_result.value, true);
}

#[test(tokio::test)]
async fn test_file_resolver_all_types() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", FLAGD_CONFIG).unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    let provider = FlagdProvider::new(FlagdOptions {
        source_configuration: Some(file_path),
        resolver_type: ResolverType::File,
        ..Default::default()
    })
    .await
    .unwrap();

    let context = EvaluationContext::default().with_targeting_key("test-user");

    let bool_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(bool_result.value, true);

    let string_result = provider
        .resolve_string_value("string-flag", &context)
        .await
        .unwrap();
    assert_eq!(string_result.value, "hello");

    let int_result = provider
        .resolve_int_value("int-flag", &context)
        .await
        .unwrap();
    assert_eq!(int_result.value, 42);

    let float_result = provider
        .resolve_float_value("float-flag", &context)
        .await
        .unwrap();
    assert_eq!(float_result.value, 3.14);

    let struct_result = provider
        .resolve_struct_value("struct-flag", &context)
        .await
        .unwrap();
    assert!(struct_result.value.fields.contains_key("key"));
    assert_eq!(
        struct_result.value.fields["key"],
        Value::String("value".to_string())
    );
}

#[test(tokio::test)]
async fn test_file_resolver_requires_source_configuration() {
    // Test that File resolver without source_configuration returns proper error
    let result = FlagdProvider::new(FlagdOptions {
        resolver_type: ResolverType::File,
        source_configuration: None, // Missing required configuration
        ..Default::default()
    })
    .await;

    assert!(
        result.is_err(),
        "Expected error when source_configuration is missing"
    );

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(
        err_msg.contains("source_configuration")
            || err_msg.contains("FLAGD_OFFLINE_FLAG_SOURCE_PATH"),
        "Error message should mention source_configuration or FLAGD_OFFLINE_FLAG_SOURCE_PATH, got: {}",
        err_msg
    );
}
