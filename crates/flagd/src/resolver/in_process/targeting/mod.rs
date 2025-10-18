use anyhow::Result;
use datalogic_rs::DataLogic;
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use serde_json::Value;
use std::sync::Arc;

mod fractional;
mod semver;

use fractional::FractionalOperator;
use semver::SemVerOperator;

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
    ) -> Result<Option<String>> {
        // Parse the rule from JSON string
        let rule_value: Value = serde_json::from_str(targeting_rule)?;

        // Compile the logic
        let compiled = self.logic.compile(&rule_value)?;

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

    // Helper to convert EvaluationContextFieldValue to serde_json::Value
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
            EvaluationContextFieldValue::Struct(s) => Value::String(format!("{:?}", s)),
        }
    }
}
