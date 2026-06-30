use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use cucumber::{World, given, then, when};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, EvaluationContextFieldValue, EvaluationReason, StructValue};
use open_feature_flagd::{CacheSettings, CacheType, FlagdOptions, FlagdProvider, ResolverType};
use serde_json::{Map, json};
use tempfile::NamedTempFile;
use test_log::test;

const EVALUATOR_REFS_FLAGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flagd-testbed/flags/evaluator-refs.json"
));
const CUSTOM_OPS_FLAGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flagd-testbed/flags/custom-ops.json"
));
const TESTING_FLAGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flagd-testbed/flags/testing-flags.json"
));
const EDGE_CASE_FLAGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flagd-testbed/flags/edge-case-flags.json"
));

#[derive(Debug, World)]
#[world(init = Self::new)]
struct TargetingWorld {
    options: FlagdOptions,
    provider: Option<FlagdProvider>,
    config_file: Option<NamedTempFile>,
    flag_key: String,
    flag_type: String,
    default_value: String,
    context: EvaluationContext,
    resolved_value: Option<String>,
    resolved_reason: Option<String>,
    resolved_error_code: Option<String>,
    resolved_variant: Option<String>,
}

impl TargetingWorld {
    fn new() -> Self {
        Self {
            options: FlagdOptions {
                resolver_type: ResolverType::File,
                cache_settings: None,
                ..Default::default()
            },
            provider: None,
            config_file: None,
            flag_key: String::new(),
            flag_type: String::new(),
            default_value: String::new(),
            context: EvaluationContext::default(),
            resolved_value: None,
            resolved_reason: None,
            resolved_error_code: None,
            resolved_variant: None,
        }
    }

    async fn clear(&mut self) {
        self.provider = None;
        self.config_file = None;
        self.options = FlagdOptions {
            resolver_type: ResolverType::File,
            cache_settings: None,
            ..Default::default()
        };
        self.flag_key.clear();
        self.flag_type.clear();
        self.default_value.clear();
        self.context = EvaluationContext::default();
        self.resolved_value = None;
        self.resolved_reason = None;
        self.resolved_error_code = None;
        self.resolved_variant = None;
    }
}

impl Default for TargetingWorld {
    fn default() -> Self {
        Self::new()
    }
}

fn targeting_flags() -> String {
    let mut root = json!({
        "$schema": "https://flagd.dev/schema/v0/flags.json",
        "flags": {}
    });

    for source in [EVALUATOR_REFS_FLAGS, CUSTOM_OPS_FLAGS, EDGE_CASE_FLAGS] {
        let source: serde_json::Value = serde_json::from_str(source).unwrap();
        merge_object(&mut root, &source, "flags");
        merge_object(&mut root, &source, "$evaluators");
    }
    merge_flags_by_name(
        &mut root,
        TESTING_FLAGS,
        &["timestamp-flag", "targeting-key-flag"],
    );

    serde_json::to_string(&root).unwrap()
}

fn merge_object(target: &mut serde_json::Value, source: &serde_json::Value, key: &str) {
    let Some(source_object) = source.get(key).and_then(|value| value.as_object()) else {
        return;
    };

    let target_object = target
        .as_object_mut()
        .unwrap()
        .entry(key)
        .or_insert_with(|| serde_json::Value::Object(Map::new()))
        .as_object_mut()
        .unwrap();

    for (name, value) in source_object {
        target_object.insert(name.clone(), value.clone());
    }
}

fn merge_flags_by_name(target: &mut serde_json::Value, source: &str, names: &[&str]) {
    let source: serde_json::Value = serde_json::from_str(source).unwrap();
    let source_flags = source
        .get("flags")
        .and_then(|value| value.as_object())
        .unwrap();
    let target_flags = target
        .get_mut("flags")
        .and_then(|value| value.as_object_mut())
        .unwrap();

    for name in names {
        target_flags.insert((*name).to_string(), source_flags[*name].clone());
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

fn context_value(type_name: &str, value: &str) -> EvaluationContextFieldValue {
    match type_name {
        "Boolean" => EvaluationContextFieldValue::Bool(value.parse().unwrap()),
        "Integer" => EvaluationContextFieldValue::Int(value.parse().unwrap()),
        "Float" => EvaluationContextFieldValue::Float(value.parse().unwrap()),
        "String" => EvaluationContextFieldValue::String(value.to_string()),
        _ => panic!("Unsupported context value type: {type_name}"),
    }
}

#[given(expr = r#"an option {string} of type {string} with value {string}"#)]
async fn option_with_value(
    world: &mut TargetingWorld,
    option: String,
    _option_type: String,
    value: String,
) {
    match option.as_str() {
        "cache" => {
            world.options.cache_settings = Some(CacheSettings {
                cache_type: match value.to_lowercase().as_str() {
                    "disabled" => CacheType::Disabled,
                    "lru" => CacheType::Lru,
                    _ => CacheType::Disabled,
                },
                ..Default::default()
            });
        }
        "resolver" => {
            world.options.resolver_type = match value.to_uppercase().as_str() {
                "FILE" => ResolverType::File,
                "IN-PROCESS" | "INPROCESS" => ResolverType::InProcess,
                "RPC" => ResolverType::Rpc,
                "REST" => ResolverType::Rest,
                _ => ResolverType::File,
            };
        }
        "deadlineMs" => {
            world.options.deadline_ms = value.parse().unwrap();
        }
        _ => {}
    }
}

#[given(expr = "a stable flagd provider")]
async fn stable_flagd_provider(world: &mut TargetingWorld) {
    let mut config_file = NamedTempFile::new().unwrap();
    config_file
        .write_all(targeting_flags().as_bytes())
        .expect("failed to write targeting flag config");

    world.options.resolver_type = ResolverType::File;
    world.options.cache_settings = None;
    world.options.source_configuration = Some(config_file.path().to_string_lossy().into_owned());

    world.provider = Some(
        FlagdProvider::new(world.options.clone())
            .await
            .expect("failed to create file resolver provider"),
    );
    world.config_file = Some(config_file);
}

#[given(regex = r#"^a ([A-Za-z]+)-flag with key "([^"]+)" and a default value "([^"]*)"$"#)]
async fn flag_with_key_and_default(
    world: &mut TargetingWorld,
    flag_type: String,
    key: String,
    default: String,
) {
    world.flag_type = flag_type;
    world.flag_key = key;
    world.default_value = default;
}

#[given(
    expr = r#"a context containing a key {string}, with type {string} and with value {string}"#
)]
async fn context_with_key(
    world: &mut TargetingWorld,
    key: String,
    type_name: String,
    value: String,
) {
    world.context = world
        .context
        .clone()
        .with_custom_field(key, context_value(&type_name, &value));
}

#[given(
    expr = r#"a context containing a nested property with outer key {string} and inner key {string}, with value {string}"#
)]
async fn context_with_nested_property(
    world: &mut TargetingWorld,
    outer_key: String,
    inner_key: String,
    value: String,
) {
    let mut fields = match world.context.custom_fields.get(&outer_key) {
        Some(EvaluationContextFieldValue::Struct(existing)) => existing
            .clone()
            .downcast::<StructValue>()
            .map(|existing| existing.fields.clone())
            .unwrap_or_default(),
        _ => HashMap::new(),
    };
    fields.insert(inner_key, open_feature::Value::String(value));
    world.context = world.context.clone().with_custom_field(
        outer_key,
        EvaluationContextFieldValue::Struct(Arc::new(StructValue { fields })),
    );
}

#[given(expr = r#"a context containing a targeting key with value {string}"#)]
async fn context_with_targeting_key(world: &mut TargetingWorld, targeting_key: String) {
    world.context = world.context.clone().with_targeting_key(targeting_key);
}

#[when(expr = "the flag was evaluated with details")]
async fn evaluate_flag_with_details(world: &mut TargetingWorld) {
    let provider = world.provider.as_ref().expect("Provider not initialized");

    match world.flag_type.as_str() {
        "Boolean" => {
            let default_bool = world.default_value.to_lowercase() == "true";
            let result = provider
                .resolve_bool_value(&world.flag_key, &world.context)
                .await;

            match result {
                Ok(details) => {
                    world.resolved_value = Some(details.value.to_string());
                    world.resolved_reason = details.reason.map(reason_to_string);
                    world.resolved_variant = details.variant;
                    world.resolved_error_code = None;
                }
                Err(err) => {
                    world.resolved_value = Some(default_bool.to_string());
                    world.resolved_reason = Some("ERROR".to_string());
                    world.resolved_error_code = Some(format!("{:?}", err.code));
                }
            }
        }
        "String" => {
            let result = provider
                .resolve_string_value(&world.flag_key, &world.context)
                .await;

            match result {
                Ok(details) => {
                    world.resolved_value = Some(details.value.clone());
                    world.resolved_reason = details.reason.map(reason_to_string);
                    world.resolved_variant = details.variant;
                    world.resolved_error_code = None;
                }
                Err(err) => {
                    world.resolved_value = Some(world.default_value.clone());
                    world.resolved_reason = Some("ERROR".to_string());
                    world.resolved_error_code = Some(format!("{:?}", err.code));
                }
            }
        }
        "Integer" => {
            let default_int = world.default_value.trim().parse::<i64>().unwrap_or(0);
            let result = provider
                .resolve_int_value(&world.flag_key, &world.context)
                .await;

            match result {
                Ok(details) => {
                    world.resolved_value = Some(details.value.to_string());
                    world.resolved_reason = details.reason.map(reason_to_string);
                    world.resolved_variant = details.variant;
                    world.resolved_error_code = None;
                }
                Err(err) => {
                    world.resolved_value = Some(default_int.to_string());
                    world.resolved_reason = Some("ERROR".to_string());
                    world.resolved_error_code = Some(format!("{:?}", err.code));
                }
            }
        }
        "Float" => {
            let default_float = world.default_value.trim().parse::<f64>().unwrap_or(0.0);
            let result = provider
                .resolve_float_value(&world.flag_key, &world.context)
                .await;

            match result {
                Ok(details) => {
                    world.resolved_value = Some(details.value.to_string());
                    world.resolved_reason = details.reason.map(reason_to_string);
                    world.resolved_variant = details.variant;
                    world.resolved_error_code = None;
                }
                Err(err) => {
                    world.resolved_value = Some(default_float.to_string());
                    world.resolved_reason = Some("ERROR".to_string());
                    world.resolved_error_code = Some(format!("{:?}", err.code));
                }
            }
        }
        _ => panic!("Unknown flag type: {}", world.flag_type),
    }
}

#[then(expr = r#"the resolved details value should be {string}"#)]
async fn resolved_value_should_be(world: &mut TargetingWorld, expected: String) {
    let actual = world
        .resolved_value
        .as_ref()
        .expect("No resolved value found");

    match world.flag_type.as_str() {
        "Boolean" => {
            assert_eq!(
                actual.parse::<bool>().unwrap(),
                expected.parse::<bool>().unwrap()
            );
        }
        "Integer" => {
            assert_eq!(
                actual.parse::<i64>().unwrap(),
                expected.parse::<i64>().unwrap()
            );
        }
        "Float" => {
            let actual = actual.parse::<f64>().unwrap();
            let expected = expected.parse::<f64>().unwrap();
            assert!((actual - expected).abs() < 0.0001);
        }
        _ => assert_eq!(actual, &expected),
    }
}

#[then(expr = r#"the reason should be {string}"#)]
async fn reason_should_be(world: &mut TargetingWorld, expected: String) {
    let actual = world
        .resolved_reason
        .as_ref()
        .expect("No resolved reason found");

    assert_eq!(actual.to_uppercase(), expected.to_uppercase());
}

#[then(expr = r#"the error-code should be {string}"#)]
async fn error_code_should_be(world: &mut TargetingWorld, expected: String) {
    if expected.is_empty() {
        assert!(
            world.resolved_error_code.is_none(),
            "Expected no error code, but got: {:?}",
            world.resolved_error_code
        );
        return;
    }

    let actual = world
        .resolved_error_code
        .as_ref()
        .expect("No error code found");

    let expected_normalized = expected.replace('_', "").to_uppercase();
    let actual_normalized = actual.replace('_', "").to_uppercase();

    assert!(
        actual_normalized.contains(&expected_normalized),
        "Error code mismatch: expected '{}' (normalized: '{}'), got '{}' (normalized: '{}')",
        expected,
        expected_normalized,
        actual,
        actual_normalized
    );
}

#[test(tokio::test)]
async fn targeting_test() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!("{}/flagd-testbed/gherkin/targeting.feature", manifest_dir);

    TargetingWorld::cucumber()
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear().await;
            })
        })
        .run_and_exit(feature_path)
        .await;
}
