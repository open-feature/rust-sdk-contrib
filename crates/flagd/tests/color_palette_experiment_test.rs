use common::{FLAGD_SYNC_PORT, Flagd};

use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, EvaluationReason};
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use std::io::Write;
use tempfile::NamedTempFile;
use test_log::test;
use testcontainers::runners::AsyncRunner;
mod common;

fn get_color_palette_experiment_config() -> &'static str {
    r#"{
        "$schema": "https://flagd.dev/schema/v0/flags.json",
        "flags": {
            "color-palette-experiment": {
                "state": "ENABLED",
                "defaultVariant": "grey",
                "variants": {
                    "red": "b91c1c",
                    "blue": "0284c7",
                    "green": "16a34a",
                    "grey": "4b5563"
                },
                "targeting": {
                    "fractional": [
                        ["red", 25],
                        ["blue", 25],
                        ["green", 25],
                        ["grey", 25]
                    ]
                }
            }
        }
    }"#
}

#[test(tokio::test)]
async fn test_color_palette_experiment_in_process() {
    let flagd = Flagd::new()
        .with_config(get_color_palette_experiment_config())
        .start()
        .await
        .unwrap();

    let port = flagd.get_host_port_ipv4(FLAGD_SYNC_PORT).await.unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::InProcess,
        ..Default::default()
    })
    .await
    .unwrap();

    let context = EvaluationContext::default().with_targeting_key("sessionId-123");

    let result = provider
        .resolve_string_value("color-palette-experiment", &context)
        .await
        .unwrap();

    // value: "#16a34a"
    // reason: "TARGETING_MATCH"
    // variant: "green"
    // flagMetadata: None
    assert_eq!(result.value, "16a34a");
    assert_eq!(result.variant, Some("green".to_string()));
    assert_eq!(result.reason.unwrap(), EvaluationReason::TargetingMatch);

    assert!(result.flag_metadata.is_none());
}

#[test(tokio::test)]
async fn test_color_palette_experiment_file_resolver() {
    // Create a temporary config file with the experimental configuration.
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", get_color_palette_experiment_config()).unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    // Create the provider under file resolver mode.
    let provider = FlagdProvider::new(FlagdOptions {
        source_configuration: Some(file_path),
        resolver_type: ResolverType::File,
        cache_settings: None,
        ..Default::default()
    })
    .await
    .unwrap();

    // Use an evaluation context with a targeting key.
    let context = EvaluationContext::default().with_targeting_key("sessionId-123");

    let result = provider
        .resolve_string_value("color-palette-experiment", &context)
        .await
        .unwrap();

    assert_eq!(result.value, "16a34a");
    assert_eq!(result.variant, Some("green".to_string()));
    assert_eq!(result.reason.unwrap(), EvaluationReason::TargetingMatch);
    assert!(result.flag_metadata.is_none());
}
