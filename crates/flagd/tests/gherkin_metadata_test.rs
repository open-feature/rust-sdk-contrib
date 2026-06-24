use std::io::Write;

use cucumber::gherkin::Step;
use cucumber::{World, given, then, when};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, FlagMetadata, FlagMetadataValue};
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use serde_json::{Map, json};
use tempfile::NamedTempFile;
use test_log::test;

const METADATA_FLAGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flagd-testbed/flags/metadata-flags.json"
));
const TESTING_FLAGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flagd-testbed/flags/testing-flags.json"
));
const COMBINED_METADATA_FLAGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flagd-testbed/flags/selector-flag-combined-metadata.json"
));

#[derive(Debug, World)]
#[world(init = Self::new)]
struct MetadataWorld {
    options: FlagdOptions,
    provider: Option<FlagdProvider>,
    config_file: Option<NamedTempFile>,
    selector: Option<String>,
    flag_key: String,
    default_value: bool,
    context: EvaluationContext,
    resolved_metadata: Option<FlagMetadata>,
}

impl MetadataWorld {
    fn new() -> Self {
        Self {
            options: FlagdOptions {
                resolver_type: ResolverType::File,
                cache_settings: None,
                ..Default::default()
            },
            provider: None,
            config_file: None,
            selector: None,
            flag_key: String::new(),
            default_value: false,
            context: EvaluationContext::default(),
            resolved_metadata: None,
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
        self.selector = None;
        self.flag_key.clear();
        self.default_value = false;
        self.context = EvaluationContext::default();
        self.resolved_metadata = None;
    }
}

impl Default for MetadataWorld {
    fn default() -> Self {
        Self::new()
    }
}

fn stable_flags() -> String {
    let mut root = json!({
        "$schema": "https://flagd.dev/schema/v0/flags.json",
        "flags": {}
    });

    merge_object(&mut root, METADATA_FLAGS, "flags");
    merge_flags_by_name(&mut root, TESTING_FLAGS, &["boolean-flag"]);

    serde_json::to_string(&root).unwrap()
}

fn merge_object(target: &mut serde_json::Value, source: &str, key: &str) {
    let source: serde_json::Value = serde_json::from_str(source).unwrap();
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

async fn create_file_provider(world: &mut MetadataWorld, config: String) {
    let mut config_file = NamedTempFile::new().unwrap();
    config_file
        .write_all(config.as_bytes())
        .expect("failed to write metadata flag config");

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

#[given(expr = r#"an option {string} of type {string} with value {string}"#)]
async fn option_with_value(
    world: &mut MetadataWorld,
    option: String,
    _option_type: String,
    value: String,
) {
    if option == "selector" {
        world.selector = Some(value);
    }
}

#[given(expr = "a stable flagd provider")]
async fn stable_flagd_provider(world: &mut MetadataWorld) {
    create_file_provider(world, stable_flags()).await;
}

#[given(expr = "a metadata flagd provider")]
async fn metadata_flagd_provider(world: &mut MetadataWorld) {
    create_file_provider(world, COMBINED_METADATA_FLAGS.to_string()).await;
}

#[given(regex = r#"^a Boolean-flag with key "([^"]+)" and a default value "([^"]*)"$"#)]
async fn boolean_flag_with_key_and_default(
    world: &mut MetadataWorld,
    key: String,
    default: String,
) {
    world.flag_key = key;
    world.default_value = default.parse().unwrap();
}

#[when(expr = "the flag was evaluated with details")]
async fn evaluate_flag_with_details(world: &mut MetadataWorld) {
    let provider = world.provider.as_ref().expect("Provider not initialized");
    let result = provider
        .resolve_bool_value(&world.flag_key, &world.context)
        .await;

    match result {
        Ok(details) => {
            world.resolved_metadata = details.flag_metadata;
        }
        Err(_) => {
            world.resolved_metadata = None;
        }
    }
}

#[then(expr = "the resolved metadata should contain")]
async fn resolved_metadata_should_contain(world: &mut MetadataWorld, #[step] step: &Step) {
    let metadata = world
        .resolved_metadata
        .as_ref()
        .expect("No resolved metadata found");
    let table = step.table().expect("Expected metadata table");

    for row in table.rows.iter().skip(1) {
        let [key, metadata_type, value] = row.as_slice() else {
            panic!("Expected metadata row with key, metadata_type, value: {row:?}");
        };

        let actual = metadata
            .values
            .get(key)
            .unwrap_or_else(|| panic!("Missing metadata key: {key}"));

        assert_metadata_value(actual, metadata_type, value);
    }
}

#[then(expr = "the resolved metadata is empty")]
async fn resolved_metadata_is_empty(world: &mut MetadataWorld) {
    assert!(
        world
            .resolved_metadata
            .as_ref()
            .is_none_or(|metadata| metadata.values.is_empty()),
        "Expected empty metadata, got: {:?}",
        world.resolved_metadata
    );
}

fn assert_metadata_value(actual: &FlagMetadataValue, metadata_type: &str, expected: &str) {
    match (metadata_type, actual) {
        ("Boolean", FlagMetadataValue::Bool(actual)) => {
            assert_eq!(*actual, expected.parse::<bool>().unwrap())
        }
        ("Integer", FlagMetadataValue::Int(actual)) => {
            assert_eq!(*actual, expected.parse::<i64>().unwrap())
        }
        ("Float", FlagMetadataValue::Float(actual)) => {
            let expected = expected.parse::<f64>().unwrap();
            assert!((*actual - expected).abs() < 0.0001);
        }
        ("String", FlagMetadataValue::String(actual)) => assert_eq!(actual, expected),
        _ => panic!(
            "Metadata value mismatch for type {metadata_type}: actual={:?}, expected={expected}",
            actual
        ),
    }
}

#[test(tokio::test)]
async fn metadata_test() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!("{}/flagd-testbed/gherkin/metadata.feature", manifest_dir);

    MetadataWorld::cucumber()
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear().await;
            })
        })
        .run_and_exit(feature_path)
        .await;
}
