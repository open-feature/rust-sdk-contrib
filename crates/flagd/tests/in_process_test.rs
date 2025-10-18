use std::time::Duration;

use common::{FLAGD_CONFIG, FLAGD_SYNC_PORT, Flagd};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, Value};
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use test_log::test;
use testcontainers::runners::AsyncRunner;

mod common;

#[test(tokio::test)]
async fn test_in_process_bool_resolution() {
    let flagd = Flagd::new()
        .with_config(FLAGD_CONFIG)
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

    let context = EvaluationContext::default().with_targeting_key("test-user");
    let bool_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();

    assert_eq!(bool_result.value, true);
}

#[test(tokio::test)]
async fn test_in_process_sync_bool_resolution() {
    let initial_config = r#"{
        "flags": {
            "sync-test-flag": {
                "state": "ENABLED",
                "variants": {
                    "on": true,
                    "off": false
                },
                "defaultVariant": "on"
            }
        }
    }"#;

    let flagd = Flagd::new().with_config(initial_config);
    let flagd_instance = flagd.clone();
    let container = flagd.start().await.unwrap();
    let port = container.get_host_port_ipv4(FLAGD_SYNC_PORT).await.unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::InProcess,
        cache_settings: None,
        ..Default::default()
    })
    .await
    .unwrap();

    let context = EvaluationContext::default();

    // Initial value check
    let initial_result = provider
        .resolve_bool_value("sync-test-flag", &context)
        .await
        .unwrap();
    assert_eq!(initial_result.value, true);

    // Update config file
    let updated_config = r#"{
        "flags": {
            "sync-test-flag": {
                "state": "ENABLED",
                "variants": {
                    "on": true,
                    "off": false
                },
                "defaultVariant": "off"
            }
        }
    }"#;
    flagd_instance.update_config(updated_config.to_string());

    // Allow time for sync
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Check updated value
    let updated_result = provider
        .resolve_bool_value("sync-test-flag", &context)
        .await
        .unwrap();
    assert_eq!(updated_result.value, false);
}

#[test(tokio::test)]
async fn test_in_process_resolver_all_types() {
    let flagd = Flagd::new()
        .with_config(FLAGD_CONFIG)
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

    let context = EvaluationContext::default();

    // Test string resolution
    let string_result = provider
        .resolve_string_value("string-flag", &context)
        .await
        .unwrap();
    assert_eq!(string_result.value, "hello");

    // Test int resolution
    let int_result = provider
        .resolve_int_value("int-flag", &context)
        .await
        .unwrap();
    assert_eq!(int_result.value, 42);

    // Test float resolution
    let float_result = provider
        .resolve_float_value("float-flag", &context)
        .await
        .unwrap();
    assert_eq!(float_result.value, 3.14);

    // Test struct resolution
    let struct_result = provider
        .resolve_struct_value("struct-flag", &context)
        .await
        .unwrap();
    assert_eq!(
        struct_result.value.fields["key"],
        Value::String("value".to_string())
    );
    assert_eq!(struct_result.value.fields["number"], Value::Int(42));
}

// TODO: MAKE THIS WORK
// #[test(tokio::test)]
// async fn test_in_process_selector() {
//     tracing_subscriber::fmt::init();
//     // Start source flagd container
//     let initial_config = r#"{
//         "$schema": "https://flagd.dev/schema/v0/flags.json",
//         "flags": {
//             "scoped-flag": {
//                 "state": "ENABLED",
//                 "variants": {
//                     "on": true,
//                     "off": false
//                 },
//                 "defaultVariant": "on",
//                 "source": "test-scope"
//             }
//         }
//     }"#;
//     let source_flagd = Flagd::new()
//         .with_config(initial_config)
//         .start()
//         .await
//         .unwrap();
//     let source_port = source_flagd.get_host_port_ipv4(FLAGD_PORT).await.unwrap();
//     debug!("Source container started on port {}", source_port);

//     let sources_config = format!(r#"[
//         {{"uri":"/etc/flagd/config.json","provider":"file"}},
//         {{"uri":"localhost:{}","provider":"grpc","selector":"test-scope"}}
//     ]"#, source_port);

//     let main_flagd = Flagd::new()
//         .with_sources(sources_config)
//         .start()
//         .await
//         .unwrap();
//     let main_port = main_flagd.get_host_port_ipv4(FLAGD_SYNC_PORT).await.unwrap();
//     sleep(Duration::from_millis(1000)).await;
//     debug!("Main container started on port {}", main_port);

//     let provider = FlagdProvider::new(FlagdOptions {
//         host: "localhost".to_string(),
//         port: main_port,
//         resolver_type: ResolverType::InProcess,
//         selector: Some("test-scope".to_string()),
//         deadline_ms: 5000, // Increase timeout to 5 seconds
//         ..Default::default()
//     })
//     .await
//     .unwrap();

//     let context = EvaluationContext::default();
//     let string_result = provider
//         .resolve_string_value("string-flag", &context)
//         .await
//         .unwrap();
//     assert_eq!(string_result.value, "hello");
// }
