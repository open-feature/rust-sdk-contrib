use cucumber::{World, given, then, when};
use open_feature::{
    EvaluationContext, EvaluationError, EvaluationErrorCode, provider::FeatureProvider,
};
use open_feature_env_var::EnvVarProvider;

test_with::runner!(env_tests);

#[test_with::module]
mod env_tests {
    use super::*;

    pub struct TestEnv {}

    impl Default for TestEnv {
        fn default() -> TestEnv {
            // SAFETY: Setting environment variables is safe here because:
            // 1. test-with::runner! ensures tests run serially (no concurrent access)
            // 2. TestEnv::drop() guarantees cleanup after each test
            // 3. These are test-specific variables that won't affect other tests
            // 4. No other threads are accessing these variables during test execution
            //
            // Note: We cannot use temp-env here because it only sets variables within
            // a closure scope, but cucumber tests need variables to persist across
            // multiple async step functions.
            unsafe {
                std::env::set_var("boolean-flag", "true");
                std::env::set_var("string-flag", "hi");
                std::env::set_var("integer-flag", "10");
                std::env::set_var("float-flag", "0.5");
            }
            TestEnv {}
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            // SAFETY: Removing environment variables is safe here because:
            // 1. We're only removing the exact variables we set in Default
            // 2. test-with::runner! ensures no concurrent test execution
            // 3. This cleanup prevents test pollution
            unsafe {
                std::env::remove_var("boolean-flag");
                std::env::remove_var("string-flag");
                std::env::remove_var("integer-flag");
                std::env::remove_var("float-flag");
            }
        }
    }

    #[derive(Debug, cucumber::World)]
    #[world(init = Self::new)]
    pub struct MyWorld {
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
    async fn when_evaluate_string_flag(
        world: &mut MyWorld,
        flag_key: String,
        _default_value: String,
    ) {
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
    async fn when_evaluate_integer_flag(
        world: &mut MyWorld,
        flag_key: String,
        _default_value: i64,
    ) {
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

    #[test_with::runtime_no_env(SKIP_CUCUMBER_TESTS)]
    fn run_cucumber_tests() {
        futures::executor::block_on(MyWorld::run("tests/features/envs/evaluation.feature"));
    }
}
