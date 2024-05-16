use cucumber::{given, then, when, World};
use open_feature::{
    provider::FeatureProvider, EvaluationContext, EvaluationError, EvaluationErrorCode,
};
use open_feature_env_var::EnvVarProvider;

#[derive(Debug, cucumber::World)]
#[world(init = Self::new)]
struct MyWorld {
    provider: EnvVarProvider,
    boolean_flag_value: bool,
    string_flag_value: String,
    integer_flag_value: i64,
    float_flag_value: f64,
    evaluation_error: EvaluationError,
}

impl MyWorld {
    fn new() -> Self {
        Self {
            provider: EnvVarProvider::default(),
            boolean_flag_value: false,
            string_flag_value: String::new(),
            integer_flag_value: 0,
            float_flag_value: 0.0,
            evaluation_error: EvaluationError {
                code: EvaluationErrorCode::General("general error".to_string()),
                message: Some("default error message".to_string()),
            },
        }
    }
}

#[given(regex = "a provider is registered")]
fn given_provider_registered(world: &mut MyWorld) {
    world.provider = EnvVarProvider::default();
    assert_eq!(
        world.provider.metadata().name,
        "Environment Variables Provider"
    );
}

#[when(regex = "a boolean flag with key \"(.*)\" is evaluated with default value \"(.*)\"")]
async fn when_evaluate_flag(world: &mut MyWorld, flag_key: String, _default_value: String) {
    world.boolean_flag_value = world
        .provider
        .resolve_bool_value(&flag_key, &EvaluationContext::default())
        .await
        .unwrap()
        .value;
}

#[then(regex = "the resolved boolean value should be \"(.*)\"")]
fn then_check_resolved_value(world: &mut MyWorld, expected_value: String) {
    assert_eq!(
        expected_value.parse::<bool>().unwrap(),
        world.boolean_flag_value
    )
}

#[when(regex = "a string flag with key \"(.*)\" is evaluated with default value \"(.*)\"")]
async fn when_evaluate_string_flag(world: &mut MyWorld, flag_key: String, _default_value: String) {
    world.string_flag_value = world
        .provider
        .resolve_string_value(&flag_key, &EvaluationContext::default())
        .await
        .unwrap()
        .value;
}

#[then(regex = "the resolved string value should be \"(.*)\"")]
fn then_check_resolved_string_value(world: &mut MyWorld, expected_value: String) {
    assert_eq!(expected_value, world.string_flag_value)
}

#[when(regex = "an integer flag with key \"(.*)\" is evaluated with default value (.*)")]
async fn when_evaluate_integer_flag(world: &mut MyWorld, flag_key: String, _default_value: i64) {
    world.integer_flag_value = world
        .provider
        .resolve_int_value(&flag_key, &EvaluationContext::default())
        .await
        .unwrap()
        .value;
}

#[then(regex = "the resolved integer value should be (.*)")]
fn then_check_resolved_integer_value(world: &mut MyWorld, expected_value: String) {
    assert_eq!(
        expected_value.parse::<i64>().unwrap(),
        world.integer_flag_value
    )
}

#[when(regex = "a float flag with key \"(.*)\" is evaluated with default value (.*)")]
async fn when_evaluate_float_flag(world: &mut MyWorld, flag_key: String, _default_value: f64) {
    world.float_flag_value = world
        .provider
        .resolve_float_value(&flag_key, &EvaluationContext::default())
        .await
        .unwrap()
        .value;
}

#[then(regex = "the resolved float value should be (.*)")]
fn then_check_resolved_float_value(world: &mut MyWorld, expected_value: String) {
    assert_eq!(
        expected_value.parse::<f64>().unwrap(),
        world.float_flag_value
    )
}

#[when(
    regex = "a boolean flag with key \"(.*)\" is evaluated with details and default value \"(.*)\""
)]
async fn when_evaluate_flag_with_details(
    world: &mut MyWorld,
    flag_key: String,
    _default_value: String,
) {
    world.boolean_flag_value = world
        .provider
        .resolve_bool_value(&flag_key, &EvaluationContext::default())
        .await
        .unwrap()
        .value;
}
#[then(
    regex = "the resolved boolean details value should be \"(.*)\", the variant should be \"(.*)\", and the reason should be \"(.*)\""
)]
fn then_check_resolved_details_value(
    world: &mut MyWorld,
    expected_value: String,
    expected_variant: String,
    expected_reason: String,
) {
    assert_eq!(
        expected_value.parse::<bool>().unwrap(),
        world.boolean_flag_value
    );
    assert_eq!(expected_variant, "Resolved");
    assert_eq!(
        expected_reason,
        "The value was resolved from the environment variable"
    );
}

#[when(
    regex = "a non-existent string flag with key \"(.*)\" is evaluated with details and a default value \"(.*)\""
)]
async fn when_evaluate_non_existent_flag_with_details(
    world: &mut MyWorld,
    flag_key: String,
    _default_value: String,
) {
    world.evaluation_error = world
        .provider
        .resolve_string_value(&flag_key, &EvaluationContext::default())
        .await
        .unwrap_err();
}

#[then(
    regex = "the reason should indicate an error and the error code should indicate a missing flag with \"(.*)\""
)]
fn then_check_resolved_non_existent_flag_with_details(
    world: &mut MyWorld,
    expected_reason: String,
) {
    assert_eq!(
        expected_reason,
        world.evaluation_error.code.to_owned().to_string()
    );
}

fn main() {
    setup();
    futures::executor::block_on(MyWorld::run("tests/features/envs/evaluation.feature"));
}

fn setup() {
    std::env::set_var("boolean-flag", "true");
    std::env::set_var("string-flag", "hi");
    std::env::set_var("integer-flag", "10");
    std::env::set_var("float-flag", "0.5");
}
