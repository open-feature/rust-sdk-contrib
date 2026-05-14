use crate::error::FlagdEvaluationError;
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use serde_json::Value;

/// JSONLogic-based targeting rule evaluator for flag evaluation.
///
/// Built-in flagd operators (`fractional`, `sem_ver`) and `ext-string`
/// operators (`ends_with`, `starts_with`, …) come from `datalogic-rs` v5
/// directly — no custom operators registered. Evaluation goes through
/// the module-level `datalogic_rs::eval_into`, which is backed by a
/// `OnceLock`-cached default engine shared across the process.
#[derive(Default)]
pub struct Operator;

impl Operator {
    pub fn new() -> Self {
        Self
    }

    pub fn apply(
        &self,
        flag_key: &str,
        targeting_rule: &str,
        ctx: &EvaluationContext,
    ) -> Result<Option<String>, FlagdEvaluationError> {
        // Parse the rule eagerly so malformed JSON surfaces as
        // FlagdEvaluationError::Json (via the From<serde_json::Error>
        // impl in error.rs) instead of being swallowed to Ok(None) by
        // the catch-all Err arm below.
        let rule: Value = serde_json::from_str(targeting_rule)?;
        let context_data = build_context(flag_key, ctx);

        match datalogic_rs::eval_into::<Value, _, _>(&rule, &context_data) {
            Ok(Value::String(s)) => Ok(Some(s)),
            Ok(Value::Null) => Ok(None),
            Ok(other) => Ok(Some(other.to_string())),
            Err(e) => {
                tracing::debug!("DataLogic evaluation error: {:?}", e);
                Ok(None)
            }
        }
    }
}

fn build_context(flag_key: &str, ctx: &EvaluationContext) -> Value {
    let mut root = serde_json::Map::new();

    if let Some(targeting_key) = &ctx.targeting_key {
        root.insert(
            "targetingKey".to_string(),
            Value::String(targeting_key.clone()),
        );
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut flagd_props = serde_json::Map::new();
    flagd_props.insert("flagKey".to_string(), Value::String(flag_key.to_string()));
    flagd_props.insert(
        "timestamp".to_string(),
        Value::Number(serde_json::Number::from(timestamp)),
    );
    root.insert("$flagd".to_string(), Value::Object(flagd_props));

    for (key, value) in &ctx.custom_fields {
        root.insert(key.clone(), evaluation_context_value_to_json(value));
    }

    Value::Object(root)
}

fn evaluation_context_value_to_json(value: &EvaluationContextFieldValue) -> Value {
    match value {
        EvaluationContextFieldValue::String(s) => Value::String(s.clone()),
        EvaluationContextFieldValue::Bool(b) => Value::Bool(*b),
        EvaluationContextFieldValue::Int(i) => Value::Number(serde_json::Number::from(*i)),
        EvaluationContextFieldValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        EvaluationContextFieldValue::DateTime(dt) => Value::String(dt.to_string()),
        EvaluationContextFieldValue::Struct(s) => s
            .downcast_ref::<open_feature::StructValue>()
            .map(struct_value_to_json)
            .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
    }
}

fn struct_value_to_json(struct_value: &open_feature::StructValue) -> Value {
    let mut map = serde_json::Map::new();
    for (key, value) in &struct_value.fields {
        map.insert(key.clone(), open_feature_value_to_json(value));
    }
    Value::Object(map)
}

fn open_feature_value_to_json(value: &open_feature::Value) -> Value {
    match value {
        open_feature::Value::String(s) => Value::String(s.clone()),
        open_feature::Value::Bool(b) => Value::Bool(*b),
        open_feature::Value::Int(i) => Value::Number(serde_json::Number::from(*i)),
        open_feature::Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        open_feature::Value::Struct(s) => struct_value_to_json(s),
        open_feature::Value::Array(arr) => {
            Value::Array(arr.iter().map(open_feature_value_to_json).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use open_feature::{EvaluationContext, StructValue, Value as OFValue};
    use std::collections::HashMap;

    #[test]
    fn test_build_context_with_targeting_key() {
        let ctx = EvaluationContext::default().with_targeting_key("user-123");

        let result = build_context("test-flag", &ctx);

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("targetingKey").unwrap(), "user-123");
        assert!(obj.contains_key("$flagd"));

        let flagd = obj.get("$flagd").unwrap().as_object().unwrap();
        assert_eq!(flagd.get("flagKey").unwrap(), "test-flag");
        assert!(flagd.contains_key("timestamp"));
    }

    #[test]
    fn test_build_context_with_custom_fields() {
        let ctx = EvaluationContext::default()
            .with_custom_field("string_field", "value")
            .with_custom_field("int_field", 42i64)
            .with_custom_field("bool_field", true)
            .with_custom_field("float_field", 3.14f64);

        let result = build_context("test-flag", &ctx);
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("string_field").unwrap(), "value");
        assert_eq!(obj.get("int_field").unwrap(), 42);
        assert_eq!(obj.get("bool_field").unwrap(), true);
        assert_eq!(obj.get("float_field").unwrap(), 3.14);
    }

    #[test]
    fn test_open_feature_value_to_json_primitives() {
        assert_eq!(
            open_feature_value_to_json(&OFValue::String("test".to_string())),
            Value::String("test".to_string())
        );
        assert_eq!(
            open_feature_value_to_json(&OFValue::Bool(true)),
            Value::Bool(true)
        );
        assert_eq!(
            open_feature_value_to_json(&OFValue::Int(42)),
            Value::Number(42.into())
        );
        assert_eq!(
            open_feature_value_to_json(&OFValue::Float(3.14)),
            Value::Number(serde_json::Number::from_f64(3.14).unwrap())
        );
    }

    #[test]
    fn test_struct_value_to_json() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), OFValue::String("test".to_string()));
        fields.insert("count".to_string(), OFValue::Int(5));
        fields.insert("enabled".to_string(), OFValue::Bool(true));

        let struct_value = StructValue { fields };
        let result = struct_value_to_json(&struct_value);

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap(), "test");
        assert_eq!(obj.get("count").unwrap(), 5);
        assert_eq!(obj.get("enabled").unwrap(), true);
    }

    #[test]
    fn test_nested_struct_value_to_json() {
        let mut inner_fields = HashMap::new();
        inner_fields.insert(
            "inner_key".to_string(),
            OFValue::String("inner_value".to_string()),
        );
        let inner_struct = StructValue {
            fields: inner_fields,
        };

        let mut outer_fields = HashMap::new();
        outer_fields.insert(
            "outer_key".to_string(),
            OFValue::String("outer_value".to_string()),
        );
        outer_fields.insert("nested".to_string(), OFValue::Struct(inner_struct));

        let outer_struct = StructValue {
            fields: outer_fields,
        };
        let result = struct_value_to_json(&outer_struct);

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("outer_key").unwrap(), "outer_value");

        let nested = obj.get("nested").unwrap().as_object().unwrap();
        assert_eq!(nested.get("inner_key").unwrap(), "inner_value");
    }

    #[test]
    fn test_array_value_to_json() {
        let array = vec![
            OFValue::String("a".to_string()),
            OFValue::Int(1),
            OFValue::Bool(true),
        ];

        let result = open_feature_value_to_json(&OFValue::Array(array));

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], "a");
        assert_eq!(arr[1], 1);
        assert_eq!(arr[2], true);
    }

    #[test]
    fn test_apply_simple_targeting_rule() {
        let operator = Operator::new();
        let ctx = EvaluationContext::default().with_custom_field("tier", "premium");

        let rule = r#"{
            "if": [
                {"==": [{"var": "tier"}, "premium"]},
                "gold",
                "silver"
            ]
        }"#;

        let result = operator.apply("test-flag", rule, &ctx).unwrap();
        assert_eq!(result, Some("gold".to_string()));
    }

    #[test]
    fn test_apply_targeting_rule_with_default() {
        let operator = Operator::new();
        let ctx = EvaluationContext::default().with_custom_field("tier", "basic");

        let rule = r#"{
            "if": [
                {"==": [{"var": "tier"}, "premium"]},
                "gold",
                "silver"
            ]
        }"#;

        let result = operator.apply("test-flag", rule, &ctx).unwrap();
        assert_eq!(result, Some("silver".to_string()));
    }

    #[test]
    fn test_apply_empty_targeting_returns_none() {
        let operator = Operator::new();
        let ctx = EvaluationContext::default();

        let rule = "null";
        let result = operator.apply("test-flag", rule, &ctx).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_apply_malformed_rule_propagates_error() {
        let operator = Operator::new();
        let ctx = EvaluationContext::default();

        let result = operator.apply("test-flag", "{ this is not json", &ctx);
        assert!(matches!(result, Err(FlagdEvaluationError::Json(_))));
    }
}
