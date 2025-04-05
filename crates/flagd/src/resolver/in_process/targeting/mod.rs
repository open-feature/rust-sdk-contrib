use anyhow::{anyhow, Result};
use datalogic_rs::datalogic::DataLogic;
use murmurhash3::murmurhash3_x86_32;
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
mod fractional;
mod semver;

use fractional::FractionalOperator;
use semver::SemVerOperator;

// Use Arc<Mutex> to make DataLogic thread-safe
pub struct Operator {
    data_logic: Arc<Mutex<DataLogic>>,
}

impl Default for Operator {
    fn default() -> Self {
        Self::new()
    }
}

impl Operator {
    pub fn new() -> Self {
        let mut data_logic = DataLogic::new();
        data_logic.register_custom_operator("fractional", Box::new(FractionalOperator));
        data_logic.register_custom_operator("semver", Box::new(SemVerOperator));
        Self {
            data_logic: Arc::new(Mutex::new(data_logic)),
        }
    }

    pub fn apply(
        &self,
        flag_key: &str,
        rule_json: &str,
        context: &EvaluationContext,
    ) -> Result<Option<String>> {
        let evaluation_context = self.convert_context_to_json(context);
        let rule: Value = serde_json::from_str(rule_json)
            .map_err(|e| anyhow!("Failed to parse rule JSON: {}", e))?;

        let data = self.build_evaluation_data(&evaluation_context, flag_key);

        // Extract the variant if it exists in the targeting rule
        let variant_from_rule = if let Value::Object(obj) = &rule {
            // Check if this is using fractional targeting
            if let Some(Value::Array(buckets)) = obj.get("fractional") {
                // This is a fractional targeting rule, find the variant we would match
                self.handle_fractional(flag_key, context, buckets)?
            } else {
                None
            }
        } else {
            None
        };

        // If we already determined a variant from custom rule parsing, return it
        if let Some(variant) = variant_from_rule {
            return Ok(Some(variant));
        }

        // Otherwise, evaluate the rule using DataLogic
        let rule_str = rule.to_string();
        let data_str =
            serde_json::to_string(&data).map_err(|e| anyhow!("Failed to serialize data: {}", e))?;

        // Lock the mutex to access data_logic
        let data_logic = self
            .data_logic
            .lock()
            .map_err(|_| anyhow!("Failed to acquire lock on DataLogic"))?;

        let result = data_logic
            .evaluate_str(&rule_str, &data_str, None)
            .map_err(|e| anyhow!("DataLogic evaluation error: {}", e))?;

        // Extract the variant from the result if it's a truthy value
        if result.as_bool().unwrap_or(false) {
            if let Some(obj) = rule.as_object() {
                if let Some(variant) = obj.get("variant").and_then(|v| v.as_str()) {
                    return Ok(Some(variant.to_string()));
                }
            }
            Ok(Some("true".to_string()))
        } else {
            Ok(None)
        }
    }

    // Directly handle fractional targeting rules
    fn handle_fractional(
        &self,
        flag_key: &str,
        context: &EvaluationContext,
        buckets: &[Value],
    ) -> Result<Option<String>> {
        // Get the targeting key
        let targeting_key = context.targeting_key.as_deref().unwrap_or("");
        let bucket_key = format!("{}{}", flag_key, targeting_key);

        // Calculate total weight and collect buckets
        let mut total_weight = 0;
        let mut bucket_variants = Vec::new();

        for bucket in buckets {
            if let Value::Array(arr) = bucket {
                if arr.len() >= 2 {
                    if let (Value::String(variant), Value::Number(weight_num)) = (&arr[0], &arr[1])
                    {
                        let weight = weight_num.as_i64().unwrap_or(1) as i32;
                        total_weight += weight;
                        bucket_variants.push((variant.clone(), weight));
                    }
                }
            }
        }

        if total_weight <= 0 {
            return Ok(None);
        }

        // Hash the bucket key
        let hash: u32 = murmurhash3_x86_32(bucket_key.as_bytes(), 0);
        let bucket = (hash as f64 / u32::MAX as f64) * 100.0;

        // Find which bucket the hash falls into
        let mut bucket_sum = 0.0;
        for (variant, weight) in bucket_variants {
            bucket_sum += (weight as f64 * 100.0) / total_weight as f64;

            if bucket < bucket_sum {
                return Ok(Some(variant));
            }
        }

        Ok(None)
    }

    fn convert_context_to_json(&self, context: &EvaluationContext) -> Value {
        let mut map = serde_json::Map::new();

        // Add targeting key if present
        if let Some(targeting_key) = &context.targeting_key {
            map.insert(
                "targetingKey".to_string(),
                Value::String(targeting_key.clone()),
            );
        }

        // Add custom fields
        for (key, value) in &context.custom_fields {
            map.insert(key.clone(), context_value_to_json(value));
        }

        Value::Object(map)
    }

    fn build_evaluation_data(
        &self,
        evaluation_context: &Value,
        flag_key: &str,
    ) -> HashMap<String, Value> {
        let mut data = HashMap::new();
        if let Value::Object(obj) = evaluation_context {
            for (key, value) in obj {
                data.insert(key.clone(), value.clone());
            }
        }
        data.insert("flagKey".to_string(), Value::String(flag_key.to_string()));
        data
    }
}

fn context_value_to_json(value: &EvaluationContextFieldValue) -> Value {
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
