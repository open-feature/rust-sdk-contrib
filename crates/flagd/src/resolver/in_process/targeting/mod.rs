use anyhow::Result;
use datalogic_rs::value::NumberValue;
use datalogic_rs::{DataLogic, DataValue};
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use serde_json::Value;
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

        // Register custom operators
        logic.register_custom_operator("fractional", Box::new(Fractional));
        logic.register_custom_operator("sem_ver", Box::new(SemVer));

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
        // Get arena from DataLogic
        let arena = logic.arena();

        // Create entries for the object
        let mut entries = Vec::new();

        // Add targeting key if present
        if let Some(targeting_key) = &ctx.targeting_key {
            let key = arena.intern_str("targetingKey");
            let value = DataValue::String(arena.intern_str(targeting_key));
            entries.push((key, value));
        }

        // Add flagd metadata
        let flagd_key = arena.intern_str("$flagd");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create flagd object entries
        let mut flagd_entries = Vec::new();
        let flag_key_str = arena.intern_str("flagKey");
        let flag_key_value = DataValue::String(arena.intern_str(flag_key));
        flagd_entries.push((flag_key_str, flag_key_value));

        let timestamp_str = arena.intern_str("timestamp");
        let timestamp_value = DataValue::Number(NumberValue::from_i64(timestamp as i64));
        flagd_entries.push((timestamp_str, timestamp_value));

        // Allocate flagd object entries in arena
        let flagd_entries_slice = arena.alloc_object_entries(&flagd_entries);
        let flagd_obj = DataValue::Object(flagd_entries_slice);
        entries.push((flagd_key, flagd_obj));

        // Add custom fields
        for (key, value) in &ctx.custom_fields {
            let key_str = arena.intern_str(key);
            let data_value = self.evaluation_context_value_to_datavalue(value, arena);
            entries.push((key_str, data_value));
        }

        // Create the final object
        let entries_slice = arena.alloc_object_entries(&entries);
        DataValue::Object(entries_slice)
    }

    // Helper to convert EvaluationContextFieldValue to DataValue
    fn evaluation_context_value_to_datavalue<'a>(
        &self,
        value: &EvaluationContextFieldValue,
        arena: &'a datalogic_rs::arena::DataArena,
    ) -> DataValue<'a> {
        match value {
            EvaluationContextFieldValue::String(s) => DataValue::String(arena.intern_str(s)),

            EvaluationContextFieldValue::Bool(b) => DataValue::Bool(*b),

            EvaluationContextFieldValue::Int(i) => DataValue::Number(NumberValue::from_i64(*i)),

            EvaluationContextFieldValue::Float(f) => DataValue::Number(NumberValue::from_f64(*f)),

            EvaluationContextFieldValue::DateTime(dt) => {
                DataValue::String(arena.intern_str(&dt.to_string()))
            }

            EvaluationContextFieldValue::Struct(s) => {
                DataValue::String(arena.intern_str(&format!("{:?}", s)))
            }
        }
    }
}
