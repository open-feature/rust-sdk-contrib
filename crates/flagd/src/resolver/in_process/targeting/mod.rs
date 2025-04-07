use anyhow::Result;
use datalogic_rs::{DataLogic, DataValue, FromJson};
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use serde_json::{json, Map, Value};
use std::sync::{Arc, Mutex};

mod fractional;
mod semver;

use fractional::Fractional;
use semver::SemVer;

pub struct Operator {
    // Wrap DataLogic in Arc<Mutex<_>> to make it thread-safe
    logic: Arc<Mutex<DataLogic>>,
}

impl Operator {
    pub fn new() -> Self {
        // Create a new DataLogic instance
        let mut logic = DataLogic::new();

        // Register custom operators using the new simple API
        logic.register_simple_operator("fractional", Fractional::fractional_op);
        logic.register_simple_operator("sem_ver", SemVer::semver_op);

        Operator {
            logic: Arc::new(Mutex::new(logic)),
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

        // Lock the mutex to access DataLogic
        let logic_instance = self
            .logic
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire lock"))?;

        // Parse the rule
        let logic = logic_instance.parse_logic_json(&rule_value, None)?;

        // Build context data directly as DataValue using JSON as intermediate
        let json_context = self.build_json_context(flag_key, ctx);
        let context_data = DataValue::from_json(&json_context, logic_instance.arena());

        // Evaluate using DataLogic
        match logic_instance.evaluate(&logic, &context_data) {
            Ok(result) => {
                // Convert result to Option<String>
                match result {
                    DataValue::String(s) => Ok(Some(s.to_string())),
                    DataValue::Null => Ok(None),
                    _ => Ok(Some(format!("{}", result))),
                }
            }
            Err(e) => {
                // Log and return None on error
                tracing::debug!("DataLogic evaluation error: {:?}", e);
                Ok(None)
            }
        }
    }

    // Build context as JSON value first
    fn build_json_context(&self, flag_key: &str, ctx: &EvaluationContext) -> Value {
        let mut data = Map::new();

        // Add targeting key if present
        if let Some(targeting_key) = &ctx.targeting_key {
            data.insert(
                "targetingKey".to_string(),
                Value::String(targeting_key.clone()),
            );
        }

        // Add flagd metadata
        let flagd_props = json!({
            "flagKey": flag_key,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        data.insert("$flagd".to_string(), flagd_props);

        // Add custom fields
        for (key, value) in &ctx.custom_fields {
            data.insert(key.clone(), self.context_value_to_json(value));
        }

        Value::Object(data)
    }

    // Helper to convert EvaluationContextFieldValue to serde_json::Value
    fn context_value_to_json(&self, value: &EvaluationContextFieldValue) -> Value {
        match value {
            EvaluationContextFieldValue::String(s) => Value::String(s.clone()),
            EvaluationContextFieldValue::Bool(b) => Value::Bool(*b),
            EvaluationContextFieldValue::Int(i) => Value::Number((*i).into()),
            EvaluationContextFieldValue::Float(f) => {
                if let Some(n) = serde_json::Number::from_f64(*f) {
                    Value::Number(n)
                } else {
                    Value::Null
                }
            }
            EvaluationContextFieldValue::DateTime(dt) => Value::String(dt.to_string()),
            EvaluationContextFieldValue::Struct(s) => Value::String(format!("{:?}", s)),
        }
    }
}
