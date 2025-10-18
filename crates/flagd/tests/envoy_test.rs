use common::{ENVOY_CONFIG, ENVOY_PORT, Envoy, FLAGD_PORT, FLAGD_SYNC_PORT, Flagd};
use open_feature::EvaluationContext;
use open_feature::provider::FeatureProvider;
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use test_log::test;
use testcontainers::ImageExt;
use testcontainers::runners::AsyncRunner;

mod common;

#[test(tokio::test)]
async fn test_envoy_name_resolver() {
    // tracing_subscriber::fmt::init();
    // Start flagd in the network with explicit port configuration
    let _flagd = Flagd::new()
        .with_network("flagd-envoy".to_string())
        .with_container_name("flagd_test_envoy_name_resolver".to_string())
        .start()
        .await
        .unwrap();

    // Build Envoy config with container name resolution
    let envoy_config = ENVOY_CONFIG;
    // Start Envoy in same network
    let envoy = Envoy::new()
        .with_config(
            envoy_config.replace("address: flagd", "address: flagd_test_envoy_name_resolver"),
        )
        .with_network("flagd-envoy".to_string())
        .with_container_name("envoy_test_envoy_name_resolver".to_string())
        .start()
        .await
        .unwrap();

    // Get mapped port for local access
    let envoy_port = envoy.get_host_port_ipv4(ENVOY_PORT).await.unwrap();

    // Configure provider with network-aware settings
    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port: envoy_port,
        target_uri: Some(format!(
            "envoy://localhost:{}/b-features-api.service",
            envoy_port
        )),
        resolver_type: ResolverType::InProcess,
        deadline_ms: 5000,
        ..Default::default()
    })
    .await
    .unwrap();

    // Test execution
    let context = EvaluationContext::default().with_targeting_key("test-user");

    let bool_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert!(bool_result.value);
}

#[test(tokio::test)]
async fn test_envoy_rpc_resolver() {
    // tracing_subscriber::fmt::init();
    // Start flagd container in the shared network
    let _flagd = Flagd::new()
        .with_network("flagd-envoy".to_string())
        .with_container_name("flagd_test_envoy_rpc_resolver")
        .start()
        .await
        .unwrap();

    // Start Envoy container in the same network using the provided envoy configuration
    let envoy = Envoy::new()
        .with_config(
            ENVOY_CONFIG
                .replace(
                    FLAGD_SYNC_PORT.to_string().as_str(),
                    FLAGD_PORT.to_string().as_str(),
                )
                .replace("address: flagd", "address: flagd_test_envoy_rpc_resolver"),
        )
        .with_network("flagd-envoy".to_string())
        .with_container_name("envoy_test_envoy_rpc_resolver".to_string())
        .start()
        .await
        .unwrap();

    // Obtain the mapped host port for Envoy
    let envoy_port = envoy.get_host_port_ipv4(ENVOY_PORT).await.unwrap();

    // Configure the provider to use the RPC resolver and an envoy-based target URI.
    // Here the target URI uses the "envoy://" scheme and specifies the desired authority.
    let options = FlagdOptions {
        host: "localhost".to_string(),
        port: envoy_port,
        target_uri: Some(format!(
            "envoy://localhost:{}/b-features-api.service",
            envoy_port
        )),
        resolver_type: ResolverType::Rpc,
        deadline_ms: 5000,
        ..Default::default()
    };

    let provider = FlagdProvider::new(options).await.unwrap();
    let context = EvaluationContext::default().with_targeting_key("test-user");

    let bool_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert!(bool_result.value);
}
