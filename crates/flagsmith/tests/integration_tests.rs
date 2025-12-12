use flagsmith::flagsmith::models::Flags;
use flagsmith_flag_engine::types::{FlagsmithValue, FlagsmithValueType};
use open_feature::provider::FeatureProvider;
use open_feature::{EvaluationContext, EvaluationReason as Reason};
use open_feature_flagsmith::{FlagsmithClient, FlagsmithProvider};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;

struct MockFlagsmithClient {
    environment_flags: Option<HashMap<String, (FlagsmithValue, bool)>>,
    identity_flags: Option<HashMap<String, (FlagsmithValue, bool)>>,
    should_error: bool,
}

impl MockFlagsmithClient {
    fn new() -> Self {
        Self {
            environment_flags: None,
            identity_flags: None,
            should_error: false,
        }
    }

    fn with_environment_flags(mut self, flags: HashMap<String, (FlagsmithValue, bool)>) -> Self {
        self.environment_flags = Some(flags);
        self
    }

    fn with_identity_flags(mut self, flags: HashMap<String, (FlagsmithValue, bool)>) -> Self {
        self.identity_flags = Some(flags);
        self
    }

    fn with_error(mut self) -> Self {
        self.should_error = true;
        self
    }

    fn build_flags(&self, flag_map: &HashMap<String, (FlagsmithValue, bool)>) -> Flags {
        let api_flags: Vec<serde_json::Value> = flag_map
            .iter()
            .map(|(name, (value, enabled))| {
                let json_value = match value.value_type {
                    FlagsmithValueType::Integer => {
                        serde_json::Value::Number(value.value.parse::<i64>().unwrap().into())
                    }
                    FlagsmithValueType::Float => serde_json::Value::Number(
                        serde_json::Number::from_f64(value.value.parse::<f64>().unwrap()).unwrap(),
                    ),
                    FlagsmithValueType::Bool => {
                        serde_json::Value::Bool(value.value.parse::<bool>().unwrap())
                    }
                    _ => serde_json::Value::String(value.value.clone()),
                };

                serde_json::json!({
                    "id": 1,
                    "feature": {
                        "id": 1,
                        "name": name,
                        "type": "STANDARD"
                    },
                    "feature_state_value": json_value,
                    "enabled": enabled,
                    "environment": 1,
                    "identity": null,
                    "feature_segment": null
                })
            })
            .collect();

        Flags::from_api_flags(&api_flags, None, None).expect("Failed to create mock flags")
    }
}

impl FlagsmithClient for MockFlagsmithClient {
    fn get_environment_flags(&self) -> Result<Flags, flagsmith::error::Error> {
        if self.should_error {
            return Err(flagsmith::error::Error::new(
                flagsmith::error::ErrorKind::FlagsmithAPIError,
                "Mock API error".to_string(),
            ));
        }

        if let Some(ref flag_map) = self.environment_flags {
            Ok(self.build_flags(flag_map))
        } else {
            Err(flagsmith::error::Error::new(
                flagsmith::error::ErrorKind::FlagsmithAPIError,
                "Not configured".to_string(),
            ))
        }
    }

    fn get_identity_flags(
        &self,
        _identifier: &str,
        _traits: Option<Vec<flagsmith::flagsmith::models::SDKTrait>>,
        _transient: Option<bool>,
    ) -> Result<Flags, flagsmith::error::Error> {
        if self.should_error {
            return Err(flagsmith::error::Error::new(
                flagsmith::error::ErrorKind::FlagsmithAPIError,
                "Mock API error".to_string(),
            ));
        }

        if let Some(ref flag_map) = self.identity_flags {
            Ok(self.build_flags(flag_map))
        } else {
            Err(flagsmith::error::Error::new(
                flagsmith::error::ErrorKind::FlagsmithAPIError,
                "Not configured".to_string(),
            ))
        }
    }
}

fn create_mock_flags(
    configs: Vec<(&str, FlagsmithValue, bool)>,
) -> HashMap<String, (FlagsmithValue, bool)> {
    configs
        .into_iter()
        .map(|(name, value, enabled)| (name.to_string(), (value, enabled)))
        .collect()
}

#[tokio::test]
async fn test_resolve_bool_value_enabled() {
    let flags = create_mock_flags(vec![(
        "test-flag",
        FlagsmithValue {
            value: "true".to_string(),
            value_type: FlagsmithValueType::Bool,
        },
        true,
    )]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_bool_value("test-flag", &context).await;

    assert!(result.is_ok());
    let details = result.unwrap();
    assert_eq!(details.value, true);
    assert_eq!(details.reason, Some(Reason::Static));
}

#[tokio::test]
async fn test_resolve_bool_value_disabled() {
    let flags = create_mock_flags(vec![(
        "test-flag",
        FlagsmithValue {
            value: "false".to_string(),
            value_type: FlagsmithValueType::Bool,
        },
        false,
    )]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_bool_value("test-flag", &context).await;

    assert!(result.is_ok());
    let details = result.unwrap();
    assert_eq!(details.value, false);
    assert_eq!(details.reason, Some(Reason::Disabled));
}

#[tokio::test]
async fn test_resolve_bool_with_targeting() {
    let flags = create_mock_flags(vec![(
        "test-flag",
        FlagsmithValue {
            value: "true".to_string(),
            value_type: FlagsmithValueType::Bool,
        },
        true,
    )]);

    let mock_client = MockFlagsmithClient::new().with_identity_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default()
        .with_targeting_key("user-123")
        .with_custom_field("email", "user@example.com");

    let result = provider.resolve_bool_value("test-flag", &context).await;

    assert!(result.is_ok());
    let details = result.unwrap();
    assert_eq!(details.value, true);
    assert_eq!(details.reason, Some(Reason::TargetingMatch));
}

#[tokio::test]
async fn test_resolve_string_value() {
    let flags = create_mock_flags(vec![(
        "color-flag",
        FlagsmithValue {
            value: "blue".to_string(),
            value_type: FlagsmithValueType::String,
        },
        true,
    )]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_string_value("color-flag", &context).await;

    assert!(result.is_ok());
    let details = result.unwrap();
    assert_eq!(details.value, "blue");
}

#[tokio::test]
async fn test_resolve_int_value() {
    let flags = create_mock_flags(vec![(
        "limit-flag",
        FlagsmithValue {
            value: "42".to_string(),
            value_type: FlagsmithValueType::Integer,
        },
        true,
    )]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_int_value("limit-flag", &context).await;

    assert!(result.is_ok());
    let details = result.unwrap();
    assert_eq!(details.value, 42);
}

#[tokio::test]
async fn test_resolve_float_value() {
    let flags = create_mock_flags(vec![(
        "rate-flag",
        FlagsmithValue {
            value: "3.14".to_string(),
            value_type: FlagsmithValueType::Float,
        },
        true,
    )]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_float_value("rate-flag", &context).await;

    assert!(result.is_ok());
    let details = result.unwrap();
    assert!((details.value - 3.14).abs() < 0.001);
}

#[tokio::test]
async fn test_resolve_struct_value() {
    let flags = create_mock_flags(vec![(
        "config-flag",
        FlagsmithValue {
            value: r#"{"name": "test", "count": 10, "active": true}"#.to_string(),
            value_type: FlagsmithValueType::String,
        },
        true,
    )]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_struct_value("config-flag", &context).await;

    assert!(result.is_ok());
    let details = result.unwrap();
    assert_eq!(details.value.fields.len(), 3);
    assert!(details.value.fields.contains_key("name"));
    assert!(details.value.fields.contains_key("count"));
    assert!(details.value.fields.contains_key("active"));
    assert_eq!(details.reason, Some(Reason::Static));
}

#[tokio::test]
async fn test_resolve_struct_value_type_mismatch() {
    // Test that resolve_struct_value rejects non-String types
    let flags = create_mock_flags(vec![(
        "int-flag",
        FlagsmithValue {
            value: "42".to_string(),
            value_type: FlagsmithValueType::Integer,
        },
        true,
    )]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_struct_value("int-flag", &context).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.code, open_feature::EvaluationErrorCode::TypeMismatch);
    assert!(
        error
            .message
            .unwrap()
            .contains("Expected string type for JSON")
    );
}

#[tokio::test]
async fn test_resolve_flag_not_found() {
    let flags = create_mock_flags(vec![]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider
        .resolve_bool_value("non-existent-flag", &context)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_resolve_type_mismatch() {
    let flags = create_mock_flags(vec![(
        "test-flag",
        FlagsmithValue {
            value: "not-a-number".to_string(),
            value_type: FlagsmithValueType::String,
        },
        true,
    )]);

    let mock_client = MockFlagsmithClient::new().with_environment_flags(flags);
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_int_value("test-flag", &context).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_resolve_api_error() {
    let mock_client = MockFlagsmithClient::new().with_error();
    let provider = FlagsmithProvider::from_client(Arc::new(mock_client));

    let context = EvaluationContext::default();
    let result = provider.resolve_bool_value("test-flag", &context).await;

    assert!(result.is_err());
}
