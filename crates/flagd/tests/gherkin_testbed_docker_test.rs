use cucumber::{World, given, then, when};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, EvaluationReason, StructValue};
use open_feature_flagd::{CacheSettings, CacheType, FlagdOptions, FlagdProvider, ResolverType};
use test_log::test;

mod common;
#[path = "common/testbed.rs"]
mod testbed;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct TestbedWorld {
    options: FlagdOptions,
    runtime: Option<testbed::RunningTestbed>,
    provider: Option<FlagdProvider>,
    flag_key: String,
    flag_type: String,
    default_value: String,
    context: EvaluationContext,
    resolved_value: Option<String>,
    resolved_reason: Option<String>,
    resolved_variant: Option<String>,
}

impl TestbedWorld {
    fn new() -> Self {
        Self {
            options: FlagdOptions::default(),
            runtime: None,
            provider: None,
            flag_key: String::new(),
            flag_type: String::new(),
            default_value: String::new(),
            context: EvaluationContext::default(),
            resolved_value: None,
            resolved_reason: None,
            resolved_variant: None,
        }
    }

    async fn clear(&mut self) {
        self.provider = None;
        self.runtime = None;
        self.options = FlagdOptions::default();
        self.flag_key.clear();
        self.flag_type.clear();
        self.default_value.clear();
        self.context = EvaluationContext::default();
        self.resolved_value = None;
        self.resolved_reason = None;
        self.resolved_variant = None;
    }
}

impl Default for TestbedWorld {
    fn default() -> Self {
        Self::new()
    }
}

fn reason_to_string(reason: EvaluationReason) -> String {
    match reason {
        EvaluationReason::Static => "STATIC".to_string(),
        EvaluationReason::TargetingMatch => "TARGETING_MATCH".to_string(),
        EvaluationReason::Default => "DEFAULT".to_string(),
        EvaluationReason::Cached => "CACHED".to_string(),
        EvaluationReason::Error => "ERROR".to_string(),
        EvaluationReason::Other(s) => s.to_uppercase(),
        _ => "UNKNOWN".to_string(),
    }
}

async fn create_provider(
    world: &mut TestbedWorld,
    resolver_type: ResolverType,
    sync_metadata: testbed::SyncMetadata,
) {
    let requested_target_uri = world.options.target_uri.clone();
    let (runtime, endpoint) = testbed::start_testbed(
        requested_target_uri.as_deref(),
        &resolver_type,
        sync_metadata,
    )
    .await;

    world.options.host = "localhost".to_string();
    world.options.resolver_type = resolver_type;
    world.options.port = endpoint.port;
    world.options.target_uri = endpoint.target_uri;

    world.provider = Some(
        FlagdProvider::new(world.options.clone())
            .await
            .expect("failed to create flagd provider"),
    );
    world.runtime = Some(runtime);
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
}

#[given(expr = r#"an option {string} of type {string} with value {string}"#)]
async fn option_with_value(
    world: &mut TestbedWorld,
    option: String,
    _option_type: String,
    value: String,
) {
    match option.as_str() {
        "cache" => {
            world.options.cache_settings = Some(CacheSettings {
                cache_type: match value.to_lowercase().as_str() {
                    "lru" => CacheType::Lru,
                    "disabled" => CacheType::Disabled,
                    _ => CacheType::Lru,
                },
                ..Default::default()
            });
        }
        "deadlineMs" => world.options.deadline_ms = value.parse().unwrap(),
        "targetUri" => world.options.target_uri = Some(value),
        _ => {}
    }
}

#[given(expr = "a stable flagd provider")]
async fn stable_flagd_provider(world: &mut TestbedWorld) {
    let resolver_type = match world.options.target_uri.as_deref() {
        Some(target_uri) if target_uri.contains("sync.service") => ResolverType::InProcess,
        _ => ResolverType::Rpc,
    };
    create_provider(world, resolver_type, testbed::SyncMetadata::Enabled).await;
}

#[given(expr = "a syncpayload flagd provider")]
async fn syncpayload_flagd_provider(world: &mut TestbedWorld) {
    create_provider(
        world,
        ResolverType::InProcess,
        testbed::SyncMetadata::Disabled,
    )
    .await;
}

#[given(regex = r#"^a ([A-Za-z]+)-flag with key "([^"]+)" and a default value "([^"]*)"$"#)]
async fn flag_with_key_and_default(
    world: &mut TestbedWorld,
    flag_type: String,
    key: String,
    default: String,
) {
    world.flag_type = flag_type;
    world.flag_key = key;
    world.default_value = default;
}

#[when(expr = "the flag was evaluated with details")]
async fn evaluate_flag_with_details(world: &mut TestbedWorld) {
    let provider = world.provider.as_ref().expect("Provider not initialized");

    match world.flag_type.as_str() {
        "Boolean" => {
            let result = provider
                .resolve_bool_value(&world.flag_key, &world.context)
                .await
                .unwrap();
            world.resolved_value = Some(result.value.to_string());
            world.resolved_reason = result.reason.map(reason_to_string);
            world.resolved_variant = result.variant;
        }
        "String" => {
            let result = provider
                .resolve_string_value(&world.flag_key, &world.context)
                .await
                .unwrap();
            world.resolved_value = Some(result.value);
            world.resolved_reason = result.reason.map(reason_to_string);
            world.resolved_variant = result.variant;
        }
        "Integer" => {
            let result = provider
                .resolve_int_value(&world.flag_key, &world.context)
                .await
                .unwrap();
            world.resolved_value = Some(result.value.to_string());
            world.resolved_reason = result.reason.map(reason_to_string);
            world.resolved_variant = result.variant;
        }
        "Float" => {
            let result = provider
                .resolve_float_value(&world.flag_key, &world.context)
                .await
                .unwrap();
            world.resolved_value = Some(result.value.to_string());
            world.resolved_reason = result.reason.map(reason_to_string);
            world.resolved_variant = result.variant;
        }
        "Object" => {
            let result = provider
                .resolve_struct_value(&world.flag_key, &world.context)
                .await
                .unwrap();
            world.resolved_value = Some(struct_value_to_json(&result.value).to_string());
            world.resolved_reason = result.reason.map(reason_to_string);
            world.resolved_variant = result.variant;
        }
        _ => panic!("Unknown flag type: {}", world.flag_type),
    }
}

fn struct_value_to_json(value: &StructValue) -> serde_json::Value {
    let map = value
        .fields
        .iter()
        .map(|(key, value)| (key.clone(), value_to_json(value)))
        .collect();
    serde_json::Value::Object(map)
}

fn value_to_json(value: &open_feature::Value) -> serde_json::Value {
    match value {
        open_feature::Value::Bool(value) => serde_json::Value::Bool(*value),
        open_feature::Value::String(value) => serde_json::Value::String(value.clone()),
        open_feature::Value::Int(value) => serde_json::Value::Number((*value).into()),
        open_feature::Value::Float(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        open_feature::Value::Struct(value) => struct_value_to_json(value),
        open_feature::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(value_to_json).collect())
        }
    }
}

#[then(expr = r#"the resolved details value should be {string}"#)]
async fn resolved_value_should_be(world: &mut TestbedWorld, expected: String) {
    compare_resolved_value(world, &expected);
}

#[then(regex = r#"^the resolved details value should be "(\{.*\})"$"#)]
async fn resolved_object_value_should_be(world: &mut TestbedWorld, expected: String) {
    compare_resolved_value(world, &expected);
}

fn compare_resolved_value(world: &TestbedWorld, expected: &str) {
    let actual = world
        .resolved_value
        .as_ref()
        .expect("No resolved value found");

    match world.flag_type.as_str() {
        "Boolean" => assert_eq!(
            actual.parse::<bool>().unwrap(),
            expected.parse::<bool>().unwrap()
        ),
        "Integer" => assert_eq!(
            actual.parse::<i64>().unwrap(),
            expected.parse::<i64>().unwrap()
        ),
        "Float" => {
            let actual = actual.parse::<f64>().unwrap();
            let expected = expected.parse::<f64>().unwrap();
            assert!((actual - expected).abs() < 0.0001);
        }
        "Object" => {
            let actual: serde_json::Value = serde_json::from_str(actual).unwrap();
            let expected: serde_json::Value = serde_json::from_str(expected).unwrap();
            assert_eq!(actual, expected);
        }
        _ => assert_eq!(actual, expected),
    }
}

#[then(expr = r#"the variant should be {string}"#)]
async fn variant_should_be(world: &mut TestbedWorld, expected: String) {
    assert_eq!(world.resolved_variant.as_deref(), Some(expected.as_str()));
}

#[then(expr = r#"the reason should be {string}"#)]
async fn reason_should_be(world: &mut TestbedWorld, expected: String) {
    assert_eq!(
        world.resolved_reason.as_deref().map(str::to_uppercase),
        Some(expected.to_uppercase())
    );
}

#[test(tokio::test)]
#[serial_test::serial]
async fn connection_stable_test() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!("{}/flagd-testbed/gherkin/connection.feature", manifest_dir);

    TestbedWorld::cucumber()
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear().await;
            })
        })
        .filter_run_and_exit(feature_path, |_feature, _rule, scenario| {
            matches!(
                scenario.name.as_str(),
                "Connection"
                    | "Connection via TargetUri rpc"
                    | "Connection via TargetUri in-process"
            ) && !scenario
                .tags
                .iter()
                .any(|tag| matches!(tag.as_str(), "customCert" | "unixsocket" | "os.linux"))
        })
        .await;
}

#[test(tokio::test)]
#[serial_test::serial]
async fn context_enrichment_basic_test() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!(
        "{}/flagd-testbed/gherkin/contextEnrichment.feature",
        manifest_dir
    );

    TestbedWorld::cucumber()
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear().await;
            })
        })
        .filter_run_and_exit(feature_path, |_feature, _rule, scenario| {
            scenario.name == "Use enriched context"
        })
        .await;
}

#[test(tokio::test)]
#[serial_test::serial]
async fn sync_payload_basic_test() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!(
        "{}/flagd-testbed/gherkin/sync-payload.feature",
        manifest_dir
    );

    TestbedWorld::cucumber()
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear().await;
            })
        })
        .filter_run_and_exit(feature_path, |_feature, _rule, scenario| {
            !scenario.tags.iter().any(|tag| tag == "grace")
        })
        .await;
}

#[test(tokio::test)]
#[serial_test::serial]
async fn rpc_caching_resolves_test() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!("{}/flagd-testbed/gherkin/rpc-caching.feature", manifest_dir);

    TestbedWorld::cucumber()
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear().await;
            })
        })
        .filter_run_and_exit(feature_path, |_feature, _rule, scenario| {
            scenario.name.starts_with("Resolves ")
        })
        .await;
}
