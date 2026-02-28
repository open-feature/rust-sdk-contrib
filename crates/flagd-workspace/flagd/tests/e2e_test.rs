use common::{FLAGD_CONFIG, FLAGD_OFREP_PORT, FLAGD_PORT, Flagd};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, Value};
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use test_log::test;
use testcontainers::runners::AsyncRunner;

mod common;

#[test(tokio::test)]
async fn test_rpc_provider() {
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
        ..Default::default()
    })
    .await
    .unwrap();

    let context = EvaluationContext::default().with_targeting_key("test-user");

    let bool_result = FeatureProvider::resolve_bool_value(&provider, "bool-flag", &context)
        .await
        .unwrap();
    assert_eq!(bool_result.value, true);

    let string_result = FeatureProvider::resolve_string_value(&provider, "string-flag", &context)
        .await
        .unwrap();
    assert_eq!(string_result.value, "hello");

    let int_result = FeatureProvider::resolve_int_value(&provider, "int-flag", &context)
        .await
        .unwrap();
    assert_eq!(int_result.value, 42);

    let float_result = FeatureProvider::resolve_float_value(&provider, "float-flag", &context)
        .await
        .unwrap();
    assert_eq!(float_result.value, 3.14);

    let struct_result = FeatureProvider::resolve_struct_value(&provider, "struct-flag", &context)
        .await
        .unwrap();
    assert!(struct_result.value.fields.contains_key("key"));
    assert_eq!(
        struct_result.value.fields["key"],
        Value::String("value".to_string())
    );
}

#[test(tokio::test)]
async fn test_rest_provider() {
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
