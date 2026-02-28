use cucumber::{World, given, then, when};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, EvaluationReason};
use open_feature_flagd::{CacheSettings, CacheType, FlagdOptions, FlagdProvider, ResolverType};
use serde_json;
use test_log::test;
use testcontainers::runners::AsyncRunner;

mod common;
use common::{FLAGD_OFREP_PORT, FLAGD_PORT, FLAGD_SYNC_PORT, Flagd};

// Merged flag configuration for evaluation testing
const EVALUATION_FLAGS: &str = r#"{
  "$schema": "https://flagd.dev/schema/v0/flags.json",
  "flags": {
    "boolean-flag": {
      "state": "ENABLED",
      "variants": {
        "on": true,
        "off": false
      },
      "defaultVariant": "on"
    },
    "string-flag": {
      "state": "ENABLED",
      "variants": {
        "greeting": "hi",
        "parting": "bye"
      },
      "defaultVariant": "greeting"
    },
    "integer-flag": {
      "state": "ENABLED",
      "variants": {
        "one": 1,
        "ten": 10
      },
      "defaultVariant": "ten"
    },
    "float-flag": {
      "state": "ENABLED",
      "variants": {
        "tenth": 0.1,
        "half": 0.5
      },
      "defaultVariant": "half"
    },
    "object-flag": {
      "state": "ENABLED",
      "variants": {
        "empty": {},
        "template": {
          "showImages": true,
          "title": "Check out these pics!",
          "imagesPerPage": 100
        }
      },
      "defaultVariant": "template"
    },
    "boolean-zero-flag": {
      "state": "ENABLED",
      "variants": {
        "zero": false,
        "non-zero": true
      },
      "defaultVariant": "zero"
    },
    "string-zero-flag": {
      "state": "ENABLED",
      "variants": {
        "zero": "",
        "non-zero": "str"
      },
      "defaultVariant": "zero"
    },
    "integer-zero-flag": {
      "state": "ENABLED",
      "variants": {
        "zero": 0,
        "non-zero": 1
      },
      "defaultVariant": "zero"
    },
    "float-zero-flag": {
      "state": "ENABLED",
      "variants": {
        "zero": 0.0,
        "non-zero": 1.0
      },
      "defaultVariant": "zero"
    },
    "boolean-targeted-zero-flag": {
      "state": "ENABLED",
      "variants": {
        "zero": false,
        "non-zero": true
      },
      "targeting": {
        "if": [
          {
            "$ref": "is_ballmer"
          },
          "zero"
        ]
      },
      "defaultVariant": "zero"
    },
    "string-targeted-zero-flag": {
      "state": "ENABLED",
      "variants": {
        "zero": "",
        "non-zero": "str"
      },
      "targeting": {
        "if": [
          {
            "$ref": "is_ballmer"
          },
          "zero"
        ]
      },
      "defaultVariant": "zero"
    },
    "integer-targeted-zero-flag": {
      "state": "ENABLED",
      "variants": {
        "zero": 0,
        "non-zero": 1
      },
      "targeting": {
        "if": [
          {
            "$ref": "is_ballmer"
          },
          "zero"
        ]
      },
      "defaultVariant": "zero"
    },
    "float-targeted-zero-flag": {
      "state": "ENABLED",
      "variants": {
        "zero": 0.0,
        "non-zero": 1.0
      },
      "targeting": {
        "if": [
          {
            "$ref": "is_ballmer"
          },
          "zero"
        ]
      },
      "defaultVariant": "zero"
    },
    "null-default-flag": {
      "state": "ENABLED",
      "variants": {
        "on": true,
        "off": false
      },
      "defaultVariant": null
    },
    "undefined-default-flag": {
      "state": "ENABLED",
      "variants": {
        "small": 10,
        "big": 1000
      }
    },
    "no-default-flag-null-targeting-variant": {
      "state": "ENABLED",
      "variants": {
        "normal": "CFO",
        "special": "CEO"
      },
      "targeting": {
        "if": [
          {
            "==": [
              "jobs@orange.com",
              {
                "var": ["email"]
              }
            ]
          },
          "special",
          null
        ]
      }
    },
    "no-default-flag-undefined-targeting-variant": {
      "state": "ENABLED",
      "variants": {
        "normal": "CFO",
        "special": "CEO"
      },
      "targeting": {
        "if": [
          {
            "==": [
              "jobs@orange.com",
              {
                "var": ["email"]
              }
            ]
          },
          "special"
        ]
      }
    }
  },
  "$evaluators": {
    "is_ballmer": {
      "==": [
        "ballmer@macrosoft.com",
        {
          "var": [
            "email"
          ]
        }
      ]
    }
  }
}"#;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct EvaluationWorld {
    options: FlagdOptions,
    provider: Option<FlagdProvider>,
    flag_key: String,
    flag_type: String,
    default_value: String,
    context: EvaluationContext,
    resolved_value: Option<String>,
    resolved_reason: Option<String>,
    resolved_error_code: Option<String>,
    resolved_variant: Option<String>,
}

// Global static container shared across all scenarios
static FLAGD_CONTAINER: std::sync::OnceLock<
    std::sync::Arc<tokio::sync::RwLock<Option<testcontainers::ContainerAsync<Flagd>>>>,
> = std::sync::OnceLock::new();
static FLAGD_PORTS: std::sync::OnceLock<std::sync::Arc<tokio::sync::RwLock<(u16, u16, u16)>>> =
    std::sync::OnceLock::new();

impl EvaluationWorld {
    fn new() -> Self {
        Self {
            options: FlagdOptions::default(),
            provider: None,
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
        // Clean up provider (but not the shared container)
        self.provider = None;

        // Reset state
        self.options = FlagdOptions::default();
        self.flag_key.clear();
        self.flag_type.clear();
        self.default_value.clear();
        self.context = EvaluationContext::default();
        self.resolved_value = None;
        self.resolved_reason = None;
        self.resolved_error_code = None;
        self.resolved_variant = None;
    }

    // Initialize the shared flagd container (called once for all scenarios)
    async fn ensure_flagd_started() -> (u16, u16, u16) {
        let container_lock =
            FLAGD_CONTAINER.get_or_init(|| std::sync::Arc::new(tokio::sync::RwLock::new(None)));

        let ports_lock =
            FLAGD_PORTS.get_or_init(|| std::sync::Arc::new(tokio::sync::RwLock::new((0, 0, 0))));

        let mut container_guard = container_lock.write().await;

        if container_guard.is_none() {
            // Start the flagd container once
            let flagd = Flagd::new()
                .with_config(EVALUATION_FLAGS)
                .start()
                .await
                .expect("Failed to start flagd container");

            let rpc_port = flagd.get_host_port_ipv4(FLAGD_PORT).await.unwrap();
            let sync_port = flagd.get_host_port_ipv4(FLAGD_SYNC_PORT).await.unwrap();
            let ofrep_port = flagd.get_host_port_ipv4(FLAGD_OFREP_PORT).await.unwrap();

            *ports_lock.write().await = (rpc_port, sync_port, ofrep_port);
            *container_guard = Some(flagd);

            // Give the container extra time to fully initialize
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        *ports_lock.read().await
    }
}

impl Default for EvaluationWorld {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to convert EvaluationReason to a string representation
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

// Helper function to convert open_feature::Value to serde_json::Value
fn value_to_json(value: &open_feature::Value) -> serde_json::Value {
    match value {
        open_feature::Value::Bool(b) => serde_json::Value::Bool(*b),
        open_feature::Value::String(s) => serde_json::Value::String(s.clone()),
        open_feature::Value::Int(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
        open_feature::Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        open_feature::Value::Array(arr) => {
            let vec: Vec<serde_json::Value> = arr.iter().map(value_to_json).collect();
            serde_json::Value::Array(vec)
        }
        open_feature::Value::Struct(s) => {
            let map: serde_json::Map<String, serde_json::Value> = s
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

#[given(expr = r#"an option {string} of type {string} with value {string}"#)]
async fn option_with_value(
    world: &mut EvaluationWorld,
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
        "resolver" => {
            world.options.resolver_type = match value.to_uppercase().as_str() {
                "RPC" => ResolverType::Rpc,
                "IN-PROCESS" | "INPROCESS" => ResolverType::InProcess,
                "FILE" => ResolverType::File,
                "REST" => ResolverType::Rest,
                _ => ResolverType::Rpc,
            };
        }
        "deadlineMs" => {
            if let Ok(deadline) = value.parse::<u32>() {
                world.options.deadline_ms = deadline;
            }
        }
        _ => {}
    }
}

#[given(expr = "a stable flagd provider")]
async fn stable_flagd_provider(world: &mut EvaluationWorld) {
    // Ensure the shared flagd container is started
    let (rpc_port, sync_port, ofrep_port) = EvaluationWorld::ensure_flagd_started().await;

    // Get the appropriate port based on resolver type
    let port = match world.options.resolver_type {
        ResolverType::Rpc => rpc_port,
        ResolverType::InProcess => sync_port,
        ResolverType::Rest => ofrep_port,
        ResolverType::File => {
            // For file mode, we don't need to connect to flagd
            0
        }
    };

    world.options.host = "localhost".to_string();
    world.options.port = port;

    // Create provider with retry logic
    let mut retry_count = 0;
    let max_retries = 3;
    let provider = loop {
        match FlagdProvider::new(world.options.clone()).await {
            Ok(p) => break p,
            Err(e) if retry_count < max_retries => {
                retry_count += 1;
                eprintln!(
                    "Provider creation failed (attempt {}/{}): {:?}, retrying...",
                    retry_count, max_retries, e
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
            Err(e) => panic!(
                "Failed to create provider after {} retries: {:?}",
                max_retries, e
            ),
        }
    };

    // Give provider time to fully initialize
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    world.provider = Some(provider);
}

#[given(regex = r#"^a ([A-Za-z]+)-flag with key "([^"]+)" and a default value "([^"]*)"$"#)]
async fn flag_with_key_and_default(
    world: &mut EvaluationWorld,
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
    world: &mut EvaluationWorld,
    key: String,
    _type_name: String,
    value: String,
) {
    world.context = world.context.clone().with_custom_field(key, value);
}

#[when(expr = "the flag was evaluated with details")]
async fn evaluate_flag_with_details(world: &mut EvaluationWorld) {
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
        "Object" => {
            let result = provider
                .resolve_struct_value(&world.flag_key, &world.context)
                .await;

            match result {
                Ok(details) => {
                    // Convert StructValue to JSON by converting fields
                    let json_obj: serde_json::Map<String, serde_json::Value> = details
                        .value
                        .fields
                        .iter()
                        .map(|(k, v)| (k.clone(), value_to_json(v)))
                        .collect();
                    let json_str =
                        serde_json::to_string(&json_obj).unwrap_or_else(|_| "{}".to_string());
                    world.resolved_value = Some(json_str);
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
        _ => panic!("Unknown flag type: {}", world.flag_type),
    }
}

#[then(expr = r#"the resolved details value should be {string}"#)]
async fn resolved_value_should_be(world: &mut EvaluationWorld, expected: String) {
    let actual = world
        .resolved_value
        .as_ref()
        .expect("No resolved value found");

    // Handle different types of comparisons
    match world.flag_type.as_str() {
        "Boolean" => {
            let expected_bool = expected.to_lowercase() == "true";
            let actual_bool = actual.to_lowercase() == "true";
            assert_eq!(
                actual_bool, expected_bool,
                "Boolean value mismatch: expected {}, got {}",
                expected, actual
            );
        }
        "String" => {
            assert_eq!(
                actual, &expected,
                "String value mismatch: expected '{}', got '{}'",
                expected, actual
            );
        }
        "Integer" => {
            let expected_int = expected.trim().parse::<i64>().unwrap_or(0);
            let actual_int = actual.trim().parse::<i64>().unwrap_or(0);
            assert_eq!(
                actual_int, expected_int,
                "Integer value mismatch: expected {}, got {}",
                expected, actual
            );
        }
        "Float" => {
            let expected_float = expected.trim().parse::<f64>().unwrap_or(0.0);
            let actual_float = actual.trim().parse::<f64>().unwrap_or(0.0);
            assert!(
                (actual_float - expected_float).abs() < 0.0001,
                "Float value mismatch: expected {}, got {}",
                expected,
                actual
            );
        }
        "Object" => {
            // Normalize JSON strings for comparison
            let expected_value: serde_json::Value =
                serde_json::from_str(&expected).unwrap_or(serde_json::json!({}));
            let actual_value: serde_json::Value =
                serde_json::from_str(actual).unwrap_or(serde_json::json!({}));
            assert_eq!(
                actual_value, expected_value,
                "Object value mismatch: expected {}, got {}",
                expected, actual
            );
        }
        _ => {
            assert_eq!(
                actual, &expected,
                "Value mismatch: expected '{}', got '{}'",
                expected, actual
            );
        }
    }
}

#[then(expr = r#"the reason should be {string}"#)]
async fn reason_should_be(world: &mut EvaluationWorld, expected: String) {
    let actual = world
        .resolved_reason
        .as_ref()
        .expect("No resolved reason found");

    assert_eq!(
        actual.to_uppercase(),
        expected.to_uppercase(),
        "Reason mismatch: expected {}, got {}",
        expected,
        actual
    );
}

#[then(expr = r#"the error-code should be {string}"#)]
async fn error_code_should_be(world: &mut EvaluationWorld, expected: String) {
    if expected.is_empty() {
        assert!(
            world.resolved_error_code.is_none(),
            "Expected no error code, but got: {:?}",
            world.resolved_error_code
        );
    } else {
        let actual = world
            .resolved_error_code
            .as_ref()
            .expect("No error code found");

        // Convert expected format (e.g., "FLAG_NOT_FOUND") to enum format (e.g., "FlagNotFound")
        let expected_normalized = expected.replace("_", "").to_uppercase();
        let actual_normalized = actual.replace("_", "").to_uppercase();

        assert!(
            actual_normalized.contains(&expected_normalized),
            "Error code mismatch: expected '{}' (normalized: '{}'), got '{}' (normalized: '{}')",
            expected,
            expected_normalized,
            actual,
            actual_normalized
        );
    }
}

#[then(expr = r#"the variant should be {string}"#)]
async fn variant_should_be(world: &mut EvaluationWorld, expected: String) {
    let actual = world
        .resolved_variant
        .as_ref()
        .expect("No resolved variant found");

    assert_eq!(
        actual, &expected,
        "Variant mismatch: expected '{}', got '{}'",
        expected, actual
    );
}

#[test(tokio::test)]
#[serial_test::serial]
async fn evaluation_test() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let feature_path = format!("{}/flagd-testbed/gherkin/evaluation.feature", manifest_dir);

    EvaluationWorld::cucumber()
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.clear().await;
            })
        })
        .run(feature_path)
        .await;
}
