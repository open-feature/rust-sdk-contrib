use std::borrow::Cow;
use std::sync::Arc;

use common::ConfigFile;
use cucumber::{World, given, then, when};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, EvaluationReason, StructValue};
use open_feature_flagd::{CacheSettings, CacheType, FlagdOptions, FlagdProvider, ResolverType};
use test_log::test;
use testcontainers::ContainerAsync;
use testcontainers::core::logs::LogSource;
use testcontainers::core::wait::LogWaitStrategy;
use testcontainers::core::{ContainerPort, Image, Mount, WaitFor};
use testcontainers::runners::AsyncRunner;

mod common;

const RPC_PORT: u16 = 8013;
const SYNC_PORT: u16 = 8015;
const OFREP_PORT: u16 = 8016;
const TESTBED_CONTEXT_VALUE: &str = r#"{"injectedmetadata":"set"}"#;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct TestbedWorld {
    options: FlagdOptions,
    container: Option<ContainerAsync<TestbedFlagd>>,
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
            container: None,
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
        self.container = None;
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

#[derive(Debug, Clone)]
struct TestbedFlagd {
    _flags_file: Arc<ConfigFile>,
    _config_file: Arc<ConfigFile>,
    mounts: Vec<Mount>,
    cmd: Vec<String>,
    exposed_ports: Vec<ContainerPort>,
}

impl TestbedFlagd {
    fn new(disable_sync_metadata: bool) -> Self {
        let flags_file = Arc::new(ConfigFile::new(testbed_flags()));
        let config_file = Arc::new(ConfigFile::new(testbed_flagd_config(disable_sync_metadata)));
        let mounts = vec![
            Mount::bind_mount(flags_file.path(), "/etc/flagd/flags.json".to_string()),
            Mount::bind_mount(config_file.path(), "/etc/flagd/config.json".to_string()),
        ];

        Self {
            _flags_file: flags_file,
            _config_file: config_file,
            mounts,
            cmd: vec![
                "start".to_string(),
                "--config".to_string(),
                "/etc/flagd/config.json".to_string(),
            ],
            exposed_ports: vec![
                ContainerPort::Tcp(RPC_PORT),
                ContainerPort::Tcp(SYNC_PORT),
                ContainerPort::Tcp(OFREP_PORT),
            ],
        }
    }
}

impl Image for TestbedFlagd {
    fn name(&self) -> &str {
        "ghcr.io/open-feature/flagd"
    }

    fn tag(&self) -> &str {
        "v0.16.0"
    }

    fn cmd(&self) -> impl IntoIterator<Item = impl Into<Cow<'_, str>>> {
        self.cmd.clone()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::Log(LogWaitStrategy::new(
                LogSource::StdErr,
                "Flag IResolver listening at [::]:8013",
            )),
            WaitFor::Log(LogWaitStrategy::new(
                LogSource::StdErr,
                "ofrep service listening at 8016",
            )),
            WaitFor::millis(100),
        ]
    }

    fn expose_ports(&self) -> &[ContainerPort] {
        &self.exposed_ports
    }

    fn mounts(&self) -> impl IntoIterator<Item = &Mount> {
        self.mounts.iter()
    }
}

fn testbed_flagd_config(disable_sync_metadata: bool) -> String {
    let mut config = serde_json::json!({
        "sources": [
            {
                "uri": "/etc/flagd/flags.json",
                "provider": "file"
            }
        ],
        "context-value": serde_json::from_str::<serde_json::Value>(TESTBED_CONTEXT_VALUE).unwrap()
    });

    if disable_sync_metadata {
        config["disable-sync-metadata"] = serde_json::Value::Bool(true);
    }

    serde_json::to_string(&config).unwrap()
}

fn testbed_flags() -> String {
    let mut testing_flags: serde_json::Value =
        serde_json::from_str(include_str!("../flagd-testbed/flags/testing-flags.json")).unwrap();
    let metadata_flags: serde_json::Value =
        serde_json::from_str(include_str!("../flagd-testbed/flags/metadata-flags.json")).unwrap();

    let testing_flags_map = testing_flags["flags"].as_object_mut().unwrap();
    testing_flags_map.extend(metadata_flags["flags"].as_object().unwrap().clone());

    serde_json::to_string(&testing_flags).unwrap()
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
    disable_sync_metadata: bool,
) {
    let container = TestbedFlagd::new(disable_sync_metadata)
        .start()
        .await
        .expect("failed to start flagd");

    world.options.host = "localhost".to_string();
    world.options.resolver_type = resolver_type.clone();
    world.options.port = match resolver_type {
        ResolverType::Rpc => container.get_host_port_ipv4(RPC_PORT).await.unwrap(),
        ResolverType::InProcess => container.get_host_port_ipv4(SYNC_PORT).await.unwrap(),
        ResolverType::Rest => panic!("REST is not used by this runner"),
        ResolverType::File => panic!("File is not used by this runner"),
    };

    world.provider = Some(
        FlagdProvider::new(world.options.clone())
            .await
            .expect("failed to create flagd provider"),
    );
    world.container = Some(container);
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
        _ => {}
    }
}

#[given(expr = "a stable flagd provider")]
async fn stable_flagd_provider(world: &mut TestbedWorld) {
    create_provider(world, ResolverType::Rpc, false).await;
}

#[given(expr = "a syncpayload flagd provider")]
async fn syncpayload_flagd_provider(world: &mut TestbedWorld) {
    create_provider(world, ResolverType::InProcess, true).await;
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
        .map(|(key, value)| {
            let value = match value {
                open_feature::Value::Bool(value) => serde_json::Value::Bool(*value),
                open_feature::Value::String(value) => serde_json::Value::String(value.clone()),
                open_feature::Value::Int(value) => serde_json::Value::Number((*value).into()),
                open_feature::Value::Float(value) => serde_json::Number::from_f64(*value)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null),
                open_feature::Value::Struct(value) => struct_value_to_json(value),
                open_feature::Value::Array(values) => serde_json::Value::Array(
                    values
                        .iter()
                        .map(|value| match value {
                            open_feature::Value::String(value) => {
                                serde_json::Value::String(value.clone())
                            }
                            _ => serde_json::Value::String(format!("{value:?}")),
                        })
                        .collect(),
                ),
            };
            (key.clone(), value)
        })
        .collect();
    serde_json::Value::Object(map)
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
            scenario.name == "Connection"
                && !scenario
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
