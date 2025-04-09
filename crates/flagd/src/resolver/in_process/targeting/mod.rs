use anyhow::Result;
use datalogic_rs::DataLogic;
use datalogic_rs::{DataValue, FromJson};
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use serde_json::Value;
use std::sync::{Arc, Mutex};

mod fractional;
mod semver;

use fractional::fractional_fn;
use semver::sem_ver_fn;

pub struct Operator {
    // Wrap DataLogic in Arc<Mutex<_>> to make it thread-safe
    logic: Arc<Mutex<DataLogic>>,
}

impl Operator {
    pub fn new() -> Self {
        // Create a new DataLogic instance
        let mut logic = DataLogic::new();

        // Register simple operators
        logic.register_simple_operator("fractional", fractional_fn);
        logic.register_simple_operator("sem_ver", sem_ver_fn);

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

        // Build context data directly as DataValue
        let context_data = self.build_datavalue_context(flag_key, ctx, &logic_instance);

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

    fn build_datavalue_context<'a>(
        &self,
        flag_key: &str,
        ctx: &EvaluationContext,
        logic: &'a DataLogic,
    ) -> DataValue<'a> {
        // Create a JSON object for our context
        let mut root = serde_json::Map::new();

        // Add targeting key if present
        if let Some(targeting_key) = &ctx.targeting_key {
            root.insert(
                "targetingKey".to_string(),
                serde_json::Value::String(targeting_key.clone()),
            );
        }

        // Add flagd metadata
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create flagd object
        let mut flagd_props = serde_json::Map::new();
        flagd_props.insert(
            "flagKey".to_string(),
            serde_json::Value::String(flag_key.to_string()),
        );
        flagd_props.insert(
            "timestamp".to_string(),
            serde_json::Value::Number(serde_json::Number::from(timestamp)),
        );

        // Add flagd object to main object
        root.insert("$flagd".to_string(), serde_json::Value::Object(flagd_props));

        // Add custom fields
        for (key, value) in &ctx.custom_fields {
            root.insert(key.clone(), self.evaluation_context_value_to_json(value));
        }

        // Create the JSON object
        let json_value = serde_json::Value::Object(root);

        // Convert JSON to DataValue using the arena
        DataValue::from_json(&json_value, logic.arena())
    }

    // Helper to convert EvaluationContextFieldValue to serde_json::Value
    fn evaluation_context_value_to_json(
        &self,
        value: &EvaluationContextFieldValue,
    ) -> serde_json::Value {
        match value {
            EvaluationContextFieldValue::String(s) => serde_json::Value::String(s.clone()),
            EvaluationContextFieldValue::Bool(b) => serde_json::Value::Bool(*b),
            EvaluationContextFieldValue::Int(i) => {
                serde_json::Value::Number(serde_json::Number::from(*i))
            }
            EvaluationContextFieldValue::Float(f) => {
                if let Some(num) = serde_json::Number::from_f64(*f) {
                    serde_json::Value::Number(num)
                } else {
                    serde_json::Value::Null
                }
            }
            EvaluationContextFieldValue::DateTime(dt) => serde_json::Value::String(dt.to_string()),
            EvaluationContextFieldValue::Struct(s) => serde_json::Value::String(format!("{:?}", s)),
        }
    }
}
