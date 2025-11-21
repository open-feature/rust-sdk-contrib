use flagsmith::{Flagsmith, FlagsmithOptions as FlagsmithSDKOptions};
use open_feature::provider::FeatureProvider;
use open_feature::{
    EvaluationContext, EvaluationContextFieldValue, EvaluationReason as Reason, StructValue, Value,
};
use open_feature_flagsmith::{FlagsmithError, FlagsmithProvider};
use std::collections::HashMap;
use std::sync::Arc;
use test_log::test;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_empty_environment_key_fails() {
    use open_feature_flagsmith::FlagsmithOptions;

    let result = FlagsmithProvider::new("".to_string(), FlagsmithOptions::default()).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        FlagsmithError::Config("Environment key cannot be empty".to_string())
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_local_evaluation_without_server_key_fails() {
    use open_feature_flagsmith::FlagsmithOptions;

    let result = FlagsmithProvider::new(
        "regular-key".to_string(),
        FlagsmithOptions::default().with_local_evaluation(true),
    )
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        FlagsmithError::Config(msg) => {
            assert!(msg.contains("server-side environment key"));
        }
        _ => panic!("Expected Config error"),
    }
}

#[test]
fn test_context_to_traits() {
    let context = EvaluationContext::default()
        .with_custom_field("email", "user@example.com")
        .with_custom_field("age", 25)
        .with_custom_field("premium", true)
        .with_custom_field("score", 98.5);

    let traits = open_feature_flagsmith::context_to_traits(&context);

    assert_eq!(traits.len(), 4);

    let trait_keys: Vec<String> = traits.iter().map(|t| t.trait_key.clone()).collect();
    assert!(trait_keys.contains(&"email".to_string()));
    assert!(trait_keys.contains(&"age".to_string()));
    assert!(trait_keys.contains(&"premium".to_string()));
    assert!(trait_keys.contains(&"score".to_string()));
}

#[test]
fn test_context_to_traits_filters_struct_fields() {
    let mut struct_fields = HashMap::new();
    struct_fields.insert(
        "nested_field".to_string(),
        Value::String("value".to_string()),
    );
    let struct_value = StructValue {
        fields: struct_fields,
    };

    let mut context = EvaluationContext::default()
        .with_custom_field("email", "user@example.com")
        .with_custom_field("age", 25);

    context.custom_fields.insert(
        "metadata".to_string(),
        EvaluationContextFieldValue::Struct(Arc::new(struct_value)),
    );

    let traits = open_feature_flagsmith::context_to_traits(&context);

    assert_eq!(traits.len(), 2);

    let trait_keys: Vec<String> = traits.iter().map(|t| t.trait_key.clone()).collect();
    assert!(trait_keys.contains(&"email".to_string()));
    assert!(trait_keys.contains(&"age".to_string()));
    assert!(!trait_keys.contains(&"metadata".to_string()));
}

#[test]
fn test_determine_reason_disabled() {
    let context = EvaluationContext::default();
    let reason = open_feature_flagsmith::determine_reason(&context, false);
    assert_eq!(reason, Reason::Disabled);
}

#[test]
fn test_determine_reason_targeting_match() {
    let context = EvaluationContext::default().with_targeting_key("user-123");
    let reason = open_feature_flagsmith::determine_reason(&context, true);
    assert_eq!(reason, Reason::TargetingMatch);
}

#[test]
fn test_determine_reason_static() {
    let context = EvaluationContext::default();
    let reason = open_feature_flagsmith::determine_reason(&context, true);
    assert_eq!(reason, Reason::Static);
}

#[test]
fn test_metadata() {
    let provider = FlagsmithProvider::from_client(Arc::new(Flagsmith::new(
        "test-key".to_string(),
        FlagsmithSDKOptions::default(),
    )));

    assert_eq!(provider.metadata().name, "flagsmith");
}

#[test]
fn test_validate_flag_key_empty() {
    let result = open_feature_flagsmith::validate_flag_key("");
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.message.unwrap().contains("empty"));
    }
}

#[test]
fn test_validate_flag_key_valid() {
    let result = open_feature_flagsmith::validate_flag_key("my-flag");
    assert!(result.is_ok());
}

#[test]
fn test_json_to_open_feature_value_primitives() {
    let json_null = serde_json::json!(null);
    let json_bool = serde_json::json!(true);
    let json_int = serde_json::json!(42);
    let json_float = serde_json::json!(3.14);
    let json_string = serde_json::json!("hello");

    assert!(matches!(
        open_feature_flagsmith::json_to_open_feature_value(json_null),
        Value::String(_)
    ));
    assert!(matches!(
        open_feature_flagsmith::json_to_open_feature_value(json_bool),
        Value::Bool(true)
    ));
    assert!(matches!(
        open_feature_flagsmith::json_to_open_feature_value(json_int),
        Value::Int(42)
    ));

    if let Value::Float(f) = open_feature_flagsmith::json_to_open_feature_value(json_float) {
        assert!((f - 3.14).abs() < 0.001);
    } else {
        panic!("Expected Float value");
    }

    if let Value::String(s) = open_feature_flagsmith::json_to_open_feature_value(json_string) {
        assert_eq!(s, "hello");
    } else {
        panic!("Expected String value");
    }
}

#[test]
fn test_json_to_open_feature_value_array() {
    let json_array = serde_json::json!([1, 2, 3]);

    if let Value::Array(arr) = open_feature_flagsmith::json_to_open_feature_value(json_array) {
        assert_eq!(arr.len(), 3);
        assert!(matches!(arr[0], Value::Int(1)));
        assert!(matches!(arr[1], Value::Int(2)));
        assert!(matches!(arr[2], Value::Int(3)));
    } else {
        panic!("Expected Array value");
    }
}

#[test]
fn test_json_to_open_feature_value_object() {
    let json_object = serde_json::json!({
        "name": "test",
        "count": 10,
        "active": true
    });

    if let Value::Struct(s) = open_feature_flagsmith::json_to_open_feature_value(json_object) {
        assert_eq!(s.fields.len(), 3);
        assert!(s.fields.contains_key("name"));
        assert!(s.fields.contains_key("count"));
        assert!(s.fields.contains_key("active"));
    } else {
        panic!("Expected Struct value");
    }
}

#[test]
fn test_json_to_open_feature_value_object_filters_nulls() {
    // Test that null values in objects are filtered out
    let json_object = serde_json::json!({
        "name": "test",
        "email": null,
        "count": 10,
        "phone": null,
        "active": true
    });

    if let Value::Struct(s) = open_feature_flagsmith::json_to_open_feature_value(json_object) {
        // Should only have 3 fields (email and phone filtered out)
        assert_eq!(s.fields.len(), 3);
        assert!(s.fields.contains_key("name"));
        assert!(s.fields.contains_key("count"));
        assert!(s.fields.contains_key("active"));
        // Null fields should not be present
        assert!(!s.fields.contains_key("email"));
        assert!(!s.fields.contains_key("phone"));
    } else {
        panic!("Expected Struct value");
    }
}

#[test]
fn test_json_to_open_feature_value_nested() {
    let json_nested = serde_json::json!({
        "user": {
            "name": "Alice",
            "age": 30
        },
        "tags": ["admin", "user"]
    });

    if let Value::Struct(s) = open_feature_flagsmith::json_to_open_feature_value(json_nested) {
        assert_eq!(s.fields.len(), 2);

        if let Some(Value::Struct(user)) = s.fields.get("user") {
            assert_eq!(user.fields.len(), 2);
        } else {
            panic!("Expected nested struct for user");
        }

        if let Some(Value::Array(tags)) = s.fields.get("tags") {
            assert_eq!(tags.len(), 2);
        } else {
            panic!("Expected array for tags");
        }
    } else {
        panic!("Expected Struct value");
    }
}
