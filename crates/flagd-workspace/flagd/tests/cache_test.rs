use common::{FLAGD_CONFIG, FLAGD_OFREP_PORT, FLAGD_PORT, FLAGD_SYNC_PORT, Flagd};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, Value};
use open_feature_flagd::{CacheSettings, CacheType, FlagdOptions, FlagdProvider, ResolverType};
use test_log::test;
use testcontainers::runners::AsyncRunner;
use tokio::time::Duration;

mod common;

#[test(tokio::test)]
async fn test_cache_ttl_and_config_change() {
    let flagd = Flagd::new()
        .with_config(FLAGD_CONFIG)
        .start()
        .await
        .unwrap();
    let port = flagd.get_host_port_ipv4(FLAGD_PORT).await.unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        target_uri: None,
        resolver_type: ResolverType::Rpc,
        cache_settings: Some(CacheSettings {
            cache_type: CacheType::Lru,
            max_size: 100,
            ttl: Some(Duration::from_secs(3)),
        }),
        ..Default::default()
    })
    .await
    .unwrap();

    let context = EvaluationContext::default();

    let initial_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(initial_result.value, true);

    drop(flagd);

    let modified_config =
        FLAGD_CONFIG.replace("\"defaultVariant\": \"on\"", "\"defaultVariant\": \"off\"");

    let new_flagd = Flagd::new()
        .with_config(modified_config)
        .start()
        .await
        .unwrap();
    let new_port = new_flagd.get_host_port_ipv4(FLAGD_PORT).await.unwrap();

    let cached_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(cached_result.value, true);

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port: new_port,
        target_uri: None,
        resolver_type: ResolverType::Rpc,
        cache_settings: Some(CacheSettings {
            cache_type: CacheType::Lru,
            max_size: 100,
            ttl: Some(Duration::from_secs(3)),
        }),
        ..Default::default()
    })
    .await
    .unwrap();

    tokio::time::sleep(Duration::from_secs(4)).await;

    let new_result = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(new_result.value, false);
}

#[test(tokio::test)]
async fn test_rpc_provider_with_lru_cache() {
    let flagd = Flagd::new()
        .with_config(FLAGD_CONFIG)
        .start()
        .await
        .unwrap();
    let port = flagd.get_host_port_ipv4(FLAGD_PORT).await.unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        target_uri: None,
        resolver_type: ResolverType::Rpc,
        cache_settings: Some(CacheSettings {
            cache_type: CacheType::Lru,
            max_size: 100,
            ttl: Some(Duration::from_secs(3)),
        }),
        ..Default::default()
    })
    .await
    .unwrap();

    verify_cache_behavior(&provider).await;
}

#[test(tokio::test)]
async fn test_rpc_provider_with_inmemory_cache() {
    let flagd = Flagd::new()
        .with_config(FLAGD_CONFIG)
        .start()
        .await
        .unwrap();
    let port = flagd.get_host_port_ipv4(FLAGD_PORT).await.unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::Rpc,
        cache_settings: Some(CacheSettings {
            cache_type: CacheType::InMemory,
            max_size: 100,
            ttl: Some(Duration::from_secs(3)),
        }),
        ..Default::default()
    })
    .await
    .unwrap();

    verify_cache_behavior(&provider).await;
}

#[test(tokio::test)]
async fn test_in_process_provider_with_lru_cache() {
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
        cache_settings: Some(CacheSettings {
            cache_type: CacheType::Lru,
            max_size: 100,
            ttl: Some(Duration::from_secs(3)),
        }),
        ..Default::default()
    })
    .await
    .unwrap();

    verify_cache_behavior(&provider).await;
}

#[test(tokio::test)]
async fn test_in_process_provider_with_inmemory_cache() {
    // tracing_subscriber::fmt::init();
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
        cache_settings: Some(CacheSettings {
            cache_type: CacheType::InMemory,
            max_size: 100,
            ttl: Some(Duration::from_secs(3)),
        }),
        ..Default::default()
    })
    .await
    .unwrap();

    verify_cache_behavior(&provider).await;
}

#[test(tokio::test)]
async fn test_rest_provider_with_lru_cache() {
    let flagd = Flagd::new()
        .with_config(FLAGD_CONFIG)
        .start()
        .await
        .unwrap();
    let port = flagd.get_host_port_ipv4(FLAGD_OFREP_PORT).await.unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::Rest,
        cache_settings: Some(CacheSettings {
            cache_type: CacheType::Lru,
            max_size: 100,
            ttl: Some(Duration::from_secs(3)),
        }),
        ..Default::default()
    })
    .await
    .unwrap();

    verify_cache_behavior(&provider).await;
}

#[test(tokio::test)]
async fn test_rest_provider_with_inmemory_cache() {
    let flagd = Flagd::new()
        .with_config(FLAGD_CONFIG)
        .start()
        .await
        .unwrap();
    let port = flagd.get_host_port_ipv4(FLAGD_OFREP_PORT).await.unwrap();

    let provider = FlagdProvider::new(FlagdOptions {
        host: "localhost".to_string(),
        port,
        resolver_type: ResolverType::Rest,
        cache_settings: Some(CacheSettings {
            cache_type: CacheType::InMemory,
            max_size: 100,
            ttl: Some(Duration::from_secs(3)),
        }),
        ..Default::default()
    })
    .await
    .unwrap();

    verify_cache_behavior(&provider).await;
}

async fn verify_cache_behavior(provider: &FlagdProvider) {
    let context = EvaluationContext::default();

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
    assert_eq!(
        struct_result.value.fields["key"],
        Value::String("value".to_string())
    );

    let cached_bool = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(cached_bool.value, true);

    tokio::time::sleep(Duration::from_secs(4)).await;

    let expired_bool = provider
        .resolve_bool_value("bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(expired_bool.value, true);
}
