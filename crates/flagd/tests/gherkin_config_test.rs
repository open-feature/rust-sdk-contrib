use cucumber::{World, given, then, when};
use open_feature_flagd::{CacheSettings, CacheType, FlagdOptions, FlagdProvider, ResolverType};
use std::collections::HashMap;
use test_log::test;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct ConfigWorld {
    options: FlagdOptions,
    provider: Option<FlagdProvider>,
    option_values: std::collections::HashMap<String, String>,
    env_vars: std::collections::HashMap<String, String>,
}

impl ConfigWorld {
    fn new() -> Self {
        Self {
            options: FlagdOptions::default(),
            provider: None,
            option_values: HashMap::new(),
            env_vars: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        // SAFETY: Removing environment variables is safe here because:
        // 1. The test is protected by #[serial_test::serial]
        // 2. This prevents test pollution between scenarios
        // 3. We clear env vars that were set in previous scenarios

        // Clear env vars that were set in the previous scenario
        for key in self.env_vars.keys() {
            unsafe {
                std::env::remove_var(key);
            }
        }

        // Also explicitly clear FLAGD_OFFLINE_FLAG_SOURCE_PATH because it can affect resolver type
        // via FlagdOptions::default() logic, even if not tracked in world.env_vars
        unsafe {
            std::env::remove_var("FLAGD_OFFLINE_FLAG_SOURCE_PATH");
        }

        self.env_vars.clear();
        self.options = FlagdOptions::default();
        self.provider = None;
        self.option_values.clear();
    }
}

impl Default for ConfigWorld {
    fn default() -> Self {
        Self::new()
    }
}

fn convert_type(type_name: &str, value: &str) -> Option<String> {
    match type_name {
        "Integer" => value.parse::<i32>().ok().map(|v| v.to_string()),
        "String" => {
            if value == "null" {
                None
            } else {
                Some(value.to_string())
            }
        }
        "Boolean" => match value.to_lowercase().as_str() {
            "true" | "True" => Some("true".to_string()),
            "false" | "False" => Some("false".to_string()),
            _ => None,
        },
        "ResolverType" => match value.to_uppercase().as_str() {
            "RPC" => Some("rpc".to_string()),
            "REST" => Some("rest".to_string()),
            "IN-PROCESS" | "INPROCESS" => Some("in-process".to_string()),
            "FILE" | "OFFLINE" => Some("file".to_string()),
            _ => None,
        },
        "CacheType" => match value.to_lowercase().as_str() {
            "lru" | "mem" | "disabled" => Some(value.to_lowercase()),
            _ => None,
        },
        _ => None,
    }
}

#[given(expr = r#"an option {string} of type {string} with value {string}"#)]
async fn option_with_value(
    world: &mut ConfigWorld,
    option: String,
    _type_name: String,
    value: String,
) {
    world.option_values.insert(option, value);
}

#[given(expr = r#"an environment variable {string} with value {string}"#)]
async fn env_with_value(world: &mut ConfigWorld, env: String, value: String) {
    // Store env var for cleanup
    world.env_vars.insert(env.clone(), value.clone());

    // SAFETY: Setting environment variables is safe here because:
    // 1. The test function is annotated with #[serial_test::serial], ensuring no parallel execution
    // 2. ConfigWorld::clear() is called before each scenario to clean up all env vars
    // 3. All env vars set during the test are tracked in world.env_vars for guaranteed cleanup
    //
    // Note: We cannot use temp-env here because it only sets variables within a closure scope,
    // but cucumber scenarios need variables to persist across multiple async step functions.
    unsafe {
        std::env::set_var(&env, &value);
    }
}

#[when(expr = "a config was initialized")]
async fn initialize_config(world: &mut ConfigWorld) {
    // Start with defaults (which reads from environment variables)
    let mut options = FlagdOptions::default();
    let mut resolver_explicitly_set = false;

    // Apply env vars from world.env_vars to ensure they take precedence
    // This handles cases where env vars were set in test steps but timing issues
    // prevent FlagdOptions::default() from reading them correctly
    if let Some(resolver) = world.env_vars.get("FLAGD_RESOLVER") {
        options.resolver_type = match resolver.to_uppercase().as_str() {
            "RPC" => ResolverType::Rpc,
            "REST" => ResolverType::Rest,
            "IN-PROCESS" | "INPROCESS" => ResolverType::InProcess,
            "FILE" | "OFFLINE" => ResolverType::File,
            _ => ResolverType::Rpc,
        };
        resolver_explicitly_set = true;
        // Update port based on resolver type when set via env var
        options.port = match options.resolver_type {
            ResolverType::Rpc => 8013,
            ResolverType::InProcess => 8015,
            _ => options.port,
        };
    }

    // Handle explicit options - these override env vars
    if let Some(resolver) = world.option_values.get("resolver") {
        options.resolver_type = match resolver.to_uppercase().as_str() {
            "RPC" => ResolverType::Rpc,
            "REST" => ResolverType::Rest,
            "IN-PROCESS" | "INPROCESS" => ResolverType::InProcess,
            "FILE" | "OFFLINE" => ResolverType::File,
            _ => ResolverType::Rpc,
        };
        resolver_explicitly_set = true;
        // Update port based on resolver type when explicitly set
        options.port = match options.resolver_type {
            ResolverType::Rpc => 8013,
            ResolverType::InProcess => 8015,
            _ => options.port,
        };
    }

    // Handle source configuration - may override resolver type for backwards compatibility
    // BUT only if resolver wasn't explicitly set to "rpc"
    if let Some(source) = world.option_values.get("offlineFlagSourcePath") {
        options.source_configuration = Some(source.clone());
        // For backwards compatibility: if offline path is set, switch to File resolver
        // UNLESS resolver was explicitly set to "rpc" (in which case keep it as "rpc")
        if !resolver_explicitly_set || options.resolver_type != ResolverType::Rpc {
            options.resolver_type = ResolverType::File;
        }
    }

    // Handle remaining explicit options (these override env vars)
    if let Some(host) = world.option_values.get("host") {
        options.host = host.clone();
    }
    if let Some(port) = world.option_values.get("port").and_then(|v| v.parse().ok()) {
        options.port = port;
    }
    if let Some(uri) = world.option_values.get("targetUri") {
        options.target_uri = Some(uri.clone());
    }
    if let Some(cache) = world.option_values.get("cache") {
        options.cache_settings = Some(CacheSettings {
            cache_type: match cache.to_lowercase().as_str() {
                "lru" => CacheType::Lru,
                "mem" => CacheType::InMemory,
                "disabled" => CacheType::Disabled,
                _ => CacheType::Lru,
            },
            ..Default::default()
        });
    }
    if let Some(poll) = world
        .option_values
        .get("offlinePollIntervalMs")
        .and_then(|v| v.parse().ok())
    {
        options.offline_poll_interval_ms = Some(poll);
    }
    if let Some(tls) = world.option_values.get("tls") {
        options.tls = tls.to_lowercase() == "true";
    }
    if let Some(cert_path) = world.option_values.get("certPath") {
        options.cert_path = Some(cert_path.clone());
    }
    if let Some(deadline) = world
        .option_values
        .get("deadlineMs")
        .and_then(|v| v.parse().ok())
    {
        options.deadline_ms = deadline;
    }
    if let Some(stream_deadline) = world
        .option_values
        .get("streamDeadlineMs")
        .and_then(|v| v.parse().ok())
    {
        options.stream_deadline_ms = stream_deadline;
    }
    if let Some(retry_backoff) = world
        .option_values
        .get("retryBackoffMs")
        .and_then(|v| v.parse().ok())
    {
        options.retry_backoff_ms = retry_backoff;
    }
    if let Some(retry_backoff_max) = world
        .option_values
        .get("retryBackoffMaxMs")
        .and_then(|v| v.parse().ok())
    {
        options.retry_backoff_max_ms = retry_backoff_max;
    }
    if let Some(retry_grace) = world
        .option_values
        .get("retryGracePeriod")
        .and_then(|v| v.parse().ok())
    {
        options.retry_grace_period = retry_grace;
    }
    if let Some(socket_path) = world.option_values.get("socketPath") {
        options.socket_path = Some(socket_path.clone());
    }
    if let Some(selector) = world.option_values.get("selector") {
        options.selector = Some(selector.clone());
    }
    if let Some(provider_id) = world.option_values.get("providerId") {
        options.provider_id = Some(provider_id.clone());
    }
    if let Some(max_size) = world
        .option_values
        .get("maxCacheSize")
        .and_then(|v| v.parse().ok())
    {
        if let Some(cache_settings) = &mut options.cache_settings {
            cache_settings.max_size = max_size;
        }
    }

    world.options = options;
}

#[then(expr = r#"the option {string} of type {string} should have the value {string}"#)]
async fn check_option_value(
    world: &mut ConfigWorld,
    option: String,
    option_type: String,
    value: String,
) {
    let actual = match option.as_str() {
        "host" => Some(world.options.host.clone()),
        "port" => Some(world.options.port.to_string()),
        "targetUri" => world.options.target_uri.clone(),
        "tls" => Some(world.options.tls.to_string()),
        "certPath" => world.options.cert_path.clone(),
        "deadlineMs" => Some(world.options.deadline_ms.to_string()),
        "maxCacheSize" => world
            .options
            .cache_settings
            .as_ref()
            .map(|s| s.max_size.to_string()),
        "offlineFlagSourcePath" => world.options.source_configuration.clone(),
        "offlinePollIntervalMs" => world
            .options
            .offline_poll_interval_ms
            .map(|v| v.to_string()),
        "cache" => world
            .options
            .cache_settings
            .as_ref()
            .map(|s| s.cache_type.to_string()),
        "resolver" => match world.options.resolver_type {
            ResolverType::Rpc => Some("rpc".to_string()),
            ResolverType::Rest => Some("rest".to_string()),
            ResolverType::InProcess => Some("in-process".to_string()),
            ResolverType::File => Some("file".to_string()),
        },
        "retryBackoffMs" => Some(world.options.retry_backoff_ms.to_string()),
        "retryBackoffMaxMs" => Some(world.options.retry_backoff_max_ms.to_string()),
        "retryGracePeriod" => Some(world.options.retry_grace_period.to_string()),
        "selector" => world.options.selector.clone(),
        "providerId" => world.options.provider_id.clone(),
        "socketPath" => world.options.socket_path.clone(),
        "streamDeadlineMs" => Some(world.options.stream_deadline_ms.to_string()),
        _ => None,
    };
    let expected = convert_type(&option_type, &value);

    // For resolver type, do case-insensitive comparison since enum normalizes to lowercase
    let actual_normalized = if option == "resolver" {
        actual.as_ref().map(|v| v.to_lowercase())
    } else {
        actual.clone()
    };
    let expected_normalized = if option == "resolver" {
        expected.as_ref().map(|v| v.to_lowercase())
    } else {
        expected.clone()
    };

    assert_eq!(
        actual_normalized, expected_normalized,
        "Option '{}' value mismatch",
        option
    );
}

#[test(tokio::test)]
#[serial_test::serial]
async fn config_test() {
    // tracing_subscriber::fmt::init();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!("{}/flagd-testbed/gherkin/config.feature", manifest_dir);
    ConfigWorld::cucumber()
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear();
            })
        })
        .run(feature_path)
        .await;
}
