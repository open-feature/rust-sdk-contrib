use std::io::Write;

use cucumber::{World, given, then, when};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, EvaluationReason};
use open_feature_flagd::{FlagdOptions, FlagdProvider, ResolverType};
use tempfile::NamedTempFile;
use test_log::test;

const SELECTOR_FLAGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flagd-testbed/flags/selector-flags.json"
));

#[derive(Debug, World)]
#[world(init = Self::new)]
struct SelectorWorld {
    options: FlagdOptions,
    provider: Option<FlagdProvider>,
    config_file: Option<NamedTempFile>,
    flag_key: String,
    flag_type: String,
    context: EvaluationContext,
    resolved_reason: Option<String>,
}

impl SelectorWorld {
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
            context: EvaluationContext::default(),
            resolved_reason: None,
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
        self.context = EvaluationContext::default();
        self.resolved_reason = None;
    }
}

impl Default for SelectorWorld {
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

#[given(expr = r#"an option {string} of type {string} with value {string}"#)]
async fn option_with_value(
    world: &mut SelectorWorld,
    option: String,
    _option_type: String,
    value: String,
) {
    if option == "selector" {
        world.options.selector = Some(value);
    }
}

#[given(expr = "a stable flagd provider")]
async fn stable_flagd_provider(world: &mut SelectorWorld) {
    let mut config_file = NamedTempFile::new().unwrap();
    config_file
        .write_all(SELECTOR_FLAGS.as_bytes())
        .expect("failed to write selector flag config");

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
    world: &mut SelectorWorld,
    flag_type: String,
    key: String,
    _default: String,
) {
    world.flag_type = flag_type;
    world.flag_key = key;
}

#[when(expr = "the flag was evaluated with details")]
async fn evaluate_flag_with_details(world: &mut SelectorWorld) {
    let provider = world.provider.as_ref().expect("Provider not initialized");

    let reason = match world.flag_type.as_str() {
        "Boolean" => provider
            .resolve_bool_value(&world.flag_key, &world.context)
            .await
            .map(|details| details.reason.map(reason_to_string)),
        "String" => provider
            .resolve_string_value(&world.flag_key, &world.context)
            .await
            .map(|details| details.reason.map(reason_to_string)),
        "Integer" => provider
            .resolve_int_value(&world.flag_key, &world.context)
            .await
            .map(|details| details.reason.map(reason_to_string)),
        "Float" => provider
            .resolve_float_value(&world.flag_key, &world.context)
            .await
            .map(|details| details.reason.map(reason_to_string)),
        _ => panic!("Unknown flag type: {}", world.flag_type),
    };

    world.resolved_reason = reason.unwrap_or_else(|_| Some("ERROR".to_string()));
}

#[then(expr = r#"the reason should be {string}"#)]
async fn reason_should_be(world: &mut SelectorWorld, expected: String) {
    let actual = world
        .resolved_reason
        .as_ref()
        .expect("No resolved reason found");

    assert_eq!(actual.to_uppercase(), expected.to_uppercase());
}

#[test(tokio::test)]
async fn selector_test() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!("{}/flagd-testbed/gherkin/selector.feature", manifest_dir);

    SelectorWorld::cucumber()
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear().await;
            })
        })
        .run_and_exit(feature_path)
        .await;
}
