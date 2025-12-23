use crate::error::FlagdError;
use datalogic_rs::DataLogic;
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use serde_json::Value;
use std::sync::Arc;

mod fractional;
mod semver;

use fractional::FractionalOperator;
use semver::SemVerOperator;

/// JSONLogic-based targeting rule evaluator for flag evaluation
///
/// Supports custom operators for flagd-specific targeting:
/// - `fractional`: Consistent hashing for percentage-based rollouts
/// - `sem_ver`: Semantic version comparison
pub struct Operator {
    logic: Arc<DataLogic>,
}

impl Default for Operator {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator {
    pub fn new() -> Self {
        // Create a new DataLogic instance
        let mut logic = DataLogic::new();

        // Register custom operators
        logic.add_operator("fractional".to_string(), Box::new(FractionalOperator));
        logic.add_operator("sem_ver".to_string(), Box::new(SemVerOperator));

        Operator {
            logic: Arc::new(logic),
        }
    }

    pub fn apply(
        &self,
        flag_key: &str,
        targeting_rule: &str,
        ctx: &EvaluationContext,
    ) -> Result<Option<String>, FlagdError> {
        // Parse the rule from JSON string
        let rule_value: Value = serde_json::from_str(targeting_rule)?;

        // Compile the logic
        let compiled = self.logic.compile(&rule_value).map_err(|e| {
            FlagdError::Provider(format!("Failed to compile targeting rule: {:?}", e))
        })?;

        // Build context data as serde_json::Value
        let context_data = Arc::new(self.build_context(flag_key, ctx));

        // Evaluate using DataLogic
        match self.logic.evaluate(&compiled, context_data) {
            Ok(result) => {
                // Convert result to Option<String>
                match result {
                    Value::String(s) => Ok(Some(s)),
                    Value::Null => Ok(None),
                    _ => Ok(Some(result.to_string())),
                }
            }
            Err(e) => {
                // Log and return None on error
                tracing::debug!("DataLogic evaluation error: {:?}", e);
                Ok(None)
            }
        }
    }

    fn build_context(&self, flag_key: &str, ctx: &EvaluationContext) -> Value {
        // Create a JSON object for our context
        let mut root = serde_json::Map::new();

        // Add targeting key if present
        if let Some(targeting_key) = &ctx.targeting_key {
            root.insert(
                "targetingKey".to_string(),
                Value::String(targeting_key.clone()),
            );
        }

        // Add flagd metadata
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create flagd object
        let mut flagd_props = serde_json::Map::new();
        flagd_props.insert("flagKey".to_string(), Value::String(flag_key.to_string()));
        flagd_props.insert(
            "timestamp".to_string(),
            Value::Number(serde_json::Number::from(timestamp)),
        );

        // Add flagd object to main object
        root.insert("$flagd".to_string(), Value::Object(flagd_props));

        // Add custom fields
        for (key, value) in &ctx.custom_fields {
            root.insert(key.clone(), self.evaluation_context_value_to_json(value));
        }

        // Return the JSON object
        Value::Object(root)
    }

    /// Convert EvaluationContextFieldValue to serde_json::Value
    fn evaluation_context_value_to_json(&self, value: &EvaluationContextFieldValue) -> Value {
        match value {
            EvaluationContextFieldValue::String(s) => Value::String(s.clone()),
            EvaluationContextFieldValue::Bool(b) => Value::Bool(*b),
            EvaluationContextFieldValue::Int(i) => Value::Number(serde_json::Number::from(*i)),
            EvaluationContextFieldValue::Float(f) => {
                if let Some(num) = serde_json::Number::from_f64(*f) {
                    Value::Number(num)
                } else {
                    Value::Null
                }
            }
            EvaluationContextFieldValue::DateTime(dt) => Value::String(dt.to_string()),
            EvaluationContextFieldValue::Struct(s) => {
                // Try to downcast to StructValue for proper serialization
                if let Some(struct_value) = s.downcast_ref::<open_feature::StructValue>() {
                    self.struct_value_to_json(struct_value)
                } else {
                    // Fallback for other types - serialize as string representation
                    Value::Object(serde_json::Map::new())
                }
            }
        }
    }

    /// Convert StructValue to serde_json::Value with proper nested serialization
    fn struct_value_to_json(&self, struct_value: &open_feature::StructValue) -> Value {
        let mut map = serde_json::Map::new();
        for (key, value) in &struct_value.fields {
            map.insert(key.clone(), self.open_feature_value_to_json(value));
        }
        Value::Object(map)
    }

    /// Convert OpenFeature Value to serde_json::Value
    fn open_feature_value_to_json(&self, value: &open_feature::Value) -> Value {
        match value {
            open_feature::Value::String(s) => Value::String(s.clone()),
            open_feature::Value::Bool(b) => Value::Bool(*b),
            open_feature::Value::Int(i) => Value::Number(serde_json::Number::from(*i)),
            open_feature::Value::Float(f) => {
                if let Some(num) = serde_json::Number::from_f64(*f) {
                    Value::Number(num)
                } else {
                    Value::Null
                }
            }
            open_feature::Value::Struct(s) => self.struct_value_to_json(s),
            open_feature::Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| self.open_feature_value_to_json(v))
                    .collect(),
            ),
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
        let operator = Operator::new();
        let ctx = EvaluationContext::default().with_targeting_key("user-123");

        let result = operator.build_context("test-flag", &ctx);

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
        let operator = Operator::new();
        let ctx = EvaluationContext::default()
            .with_custom_field("string_field", "value")
            .with_custom_field("int_field", 42i64)
            .with_custom_field("bool_field", true)
            .with_custom_field("float_field", 3.14f64);

        let result = operator.build_context("test-flag", &ctx);
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("string_field").unwrap(), "value");
        assert_eq!(obj.get("int_field").unwrap(), 42);
        assert_eq!(obj.get("bool_field").unwrap(), true);
        assert_eq!(obj.get("float_field").unwrap(), 3.14);
    }

    #[test]
    fn test_open_feature_value_to_json_primitives() {
        let operator = Operator::new();

        assert_eq!(
            operator.open_feature_value_to_json(&OFValue::String("test".to_string())),
            Value::String("test".to_string())
        );
        assert_eq!(
            operator.open_feature_value_to_json(&OFValue::Bool(true)),
            Value::Bool(true)
        );
        assert_eq!(
            operator.open_feature_value_to_json(&OFValue::Int(42)),
            Value::Number(42.into())
        );
        assert_eq!(
            operator.open_feature_value_to_json(&OFValue::Float(3.14)),
            Value::Number(serde_json::Number::from_f64(3.14).unwrap())
        );
    }

    #[test]
    fn test_struct_value_to_json() {
        let operator = Operator::new();

        let mut fields = HashMap::new();
        fields.insert("name".to_string(), OFValue::String("test".to_string()));
        fields.insert("count".to_string(), OFValue::Int(5));
        fields.insert("enabled".to_string(), OFValue::Bool(true));

        let struct_value = StructValue { fields };
        let result = operator.struct_value_to_json(&struct_value);

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap(), "test");
        assert_eq!(obj.get("count").unwrap(), 5);
        assert_eq!(obj.get("enabled").unwrap(), true);
    }

    #[test]
    fn test_nested_struct_value_to_json() {
        let operator = Operator::new();

        // Create nested struct
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
        let result = operator.struct_value_to_json(&outer_struct);

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("outer_key").unwrap(), "outer_value");

        let nested = obj.get("nested").unwrap().as_object().unwrap();
        assert_eq!(nested.get("inner_key").unwrap(), "inner_value");
    }

    #[test]
    fn test_array_value_to_json() {
        let operator = Operator::new();

        let array = vec![
            OFValue::String("a".to_string()),
            OFValue::Int(1),
            OFValue::Bool(true),
        ];

        let result = operator.open_feature_value_to_json(&OFValue::Array(array));

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

        // Simple if rule: if tier == "premium" then "gold" else "silver"
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
}
