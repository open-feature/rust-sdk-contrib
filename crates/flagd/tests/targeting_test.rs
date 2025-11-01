use common::{FLAGD_OFREP_PORT, FLAGD_PORT, FLAGD_SYNC_PORT, Flagd};
use open_feature::EvaluationContext;
use open_feature::provider::FeatureProvider;
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use std::io::Write;
use tempfile::NamedTempFile;
use test_log::test;
use testcontainers::runners::AsyncRunner;
use tracing::debug;

mod common;

fn get_targeting_test_config() -> &'static str {
    r#"{
        "$schema": "https://flagd.dev/schema/v0/flags.json",
        "flags": {
            "fractional-flag": {
                "state": "ENABLED",
                "variants": {
                    "red": "red-value",
                    "blue": "blue-value"
                },
                "defaultVariant": "red",
                "targeting": {
                    "fractional": [
                        { "cat": [
                            { "var": "$flagd.flagKey" },
                            { "var": "email" }
                        ]},
                        ["red", 30],
                        ["blue", 70]
                    ]
                }
            },
            "semver-flag": {
                "state": "ENABLED",
                "variants": {
                    "new": "new-value",
                    "old": "old-value"
                },
                "defaultVariant": "old",
                "targeting": {
                    "if": [
                        {
                            "sem_ver": [{"var": "version"}, ">=", "2.0.0"]
                        },
                        "new",
                        "old"
                    ]
                }
            },
            "email-domain-flag": {
                "state": "ENABLED",
                "variants": {
                    "internal": true,
                    "external": false
                },
                "defaultVariant": "external",
                "targeting": {
                    "if": [
                        {
                            "ends_with": [{"var": "email"}, "@company.com"]
                        },
                        "internal",
                        "external"
                    ]
                }
            }
        }
    }"#
}

async fn verify_targeting_rules(provider: &FlagdProvider) {
    // Detailed test for fractional distribution:
    let iterations = 100;
    let mut blue_count = 0;

    for i in 0..iterations {
        // Use a different email each iteration to generate different bucket values
        let email = format!("user{}@example.com", i);
        let context = EvaluationContext::default()
            .with_targeting_key("user-1")
            .with_custom_field("email", email);
        let result = provider
            .resolve_string_value("fractional-flag", &context)
            .await
            .unwrap();

        // Count occurrence for blue variant
        if result.value == "blue-value" {
            blue_count += 1;
        }
    }

    assert!(
        blue_count >= 65,
        "Expected at least 65 blue outcomes, but got {}",
        blue_count
    );

    // Test semantic version targeting
    let context = EvaluationContext::default().with_custom_field("version", "2.1.0");
    let result = provider
        .resolve_string_value("semver-flag", &context)
        .await
        .unwrap();
    assert_eq!(result.value, "new-value");

    let context_old_version = EvaluationContext::default().with_custom_field("version", "1.9.0");
    let result = provider
        .resolve_string_value("semver-flag", &context_old_version)
        .await
        .unwrap();
    assert_eq!(result.value, "old-value");

    // Test email domain targeting
    let context = EvaluationContext::default().with_custom_field("email", "employee@company.com");
    let result = provider
        .resolve_bool_value("email-domain-flag", &context)
        .await
        .unwrap();
    assert_eq!(result.value, true);

    let context_external =
        EvaluationContext::default().with_custom_field("email", "user@external.com");
    let result = provider
        .resolve_bool_value("email-domain-flag", &context_external)
        .await
        .unwrap();
    assert_eq!(result.value, false);
}

#[test(tokio::test)]
async fn test_targeting_rules_rpc() {
    let flagd = Flagd::new()
        .with_config(get_targeting_test_config())
        .start()
        .await
        .unwrap();

    let port = flagd.get_host_port_ipv4(FLAGD_PORT).await.unwrap();
    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::Rpc,
        ..Default::default()
    })
    .await
    .unwrap();

    verify_targeting_rules(&provider).await;
}

#[test(tokio::test)]
async fn test_targeting_rules_rest() {
    let flagd = Flagd::new()
        .with_config(get_targeting_test_config())
        .start()
        .await
        .unwrap();

    let port = flagd.get_host_port_ipv4(FLAGD_OFREP_PORT).await.unwrap();
    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::Rest,
        cache_settings: None,
        ..Default::default()
    })
    .await
    .unwrap();

    verify_targeting_rules(&provider).await;
}

#[test(tokio::test)]
async fn test_targeting_rules_in_process() {
    let flagd = Flagd::new()
        .with_config(get_targeting_test_config())
        .start()
        .await
        .unwrap();

    let port = flagd.get_host_port_ipv4(FLAGD_SYNC_PORT).await.unwrap();
    debug!("Using SYNC port: {}", port);

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::InProcess,
        ..Default::default()
    })
    .await
    .unwrap();

    verify_targeting_rules(&provider).await;
}

#[test(tokio::test)]
async fn test_targeting_rules_in_process_no_cache() {
    let flagd = Flagd::new()
        .with_config(get_targeting_test_config())
        .start()
        .await
        .unwrap();

    let port = flagd.get_host_port_ipv4(FLAGD_SYNC_PORT).await.unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::InProcess,
        cache_settings: None,
        ..Default::default()
    })
    .await
    .unwrap();

    verify_targeting_rules(&provider).await;
}

#[test(tokio::test)]
async fn test_targeting_rules_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", get_targeting_test_config()).unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    let provider = FlagdProvider::new(FlagdOptions {
        source_configuration: Some(file_path),
        resolver_type: ResolverType::File,
        ..Default::default()
    })
    .await
    .unwrap();

    verify_targeting_rules(&provider).await;
}
