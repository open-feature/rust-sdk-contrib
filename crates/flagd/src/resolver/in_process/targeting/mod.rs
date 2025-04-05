use datalogic_rs::datalogic::DataLogic;
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use std::collections::HashMap;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
mod fractional;
mod semver;

use fractional::FractionalOperator;
use semver::SemVerOperator;

// Use Arc<Mutex> to make DataLogic thread-safe
pub struct Operator {
    data_logic: Arc<Mutex<DataLogic>>,
}

impl Operator {
    pub fn new() -> Self {
        let mut data_logic = DataLogic::new();
        data_logic.register_custom_operator("fractional", Box::new(FractionalOperator));
        data_logic.register_custom_operator("semver", Box::new(SemVerOperator));
        Self { data_logic: Arc::new(Mutex::new(data_logic)) }
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
        
        // Use evaluate_str method for DataLogic
        let rule_str = rule.to_string();
        let data_str = serde_json::to_string(&data)
            .map_err(|e| anyhow!("Failed to serialize data: {}", e))?;
        
        // Lock the mutex to access data_logic
        let data_logic = self.data_logic.lock()
            .map_err(|_| anyhow!("Failed to acquire lock on DataLogic"))?;
        
        let result = data_logic.evaluate_str(&rule_str, &data_str, None)
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

    fn convert_context_to_json(&self, context: &EvaluationContext) -> Value {
        let mut map = serde_json::Map::new();
        
        // Add targeting key if present
        if let Some(targeting_key) = &context.targeting_key {
            map.insert("targetingKey".to_string(), Value::String(targeting_key.clone()));
        }
        
        // Add custom fields
        for (key, value) in &context.custom_fields {
            map.insert(key.clone(), context_value_to_json(value));
        }
        
        Value::Object(map)
    }

    fn build_evaluation_data(&self, evaluation_context: &Value, flag_key: &str) -> HashMap<String, Value> {
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
