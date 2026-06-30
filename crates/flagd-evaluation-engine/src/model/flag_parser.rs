use super::feature_flag::FeatureFlag;
use super::feature_flag::ParsingResult;
use crate::error::FlagdEvaluationError;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct FlagParser;

impl FlagParser {
    pub fn parse_string(configuration: &str) -> Result<ParsingResult, FlagdEvaluationError> {
        let mut value: Value = serde_json::from_str(configuration)?;
        Self::transpose_evaluator_refs(&mut value)?;

        let obj = value
            .as_object()
            .ok_or_else(|| FlagdEvaluationError::Parse("Invalid JSON structure".to_string()))?;

        let flags = obj
            .get("flags")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                FlagdEvaluationError::Parse(
                    "No flag configurations found in the payload".to_string(),
                )
            })?;

        let flag_set_metadata = obj
            .get("metadata")
            .and_then(|v| v.as_object())
            .map(Self::convert_map_to_hashmap)
            .unwrap_or_default();

        let mut flag_map = HashMap::new();
        for (key, value) in flags {
            let flag: FeatureFlag = serde_json::from_value(value.clone())?;
            flag_map.insert(key.clone(), flag);
        }

        Ok(ParsingResult {
            flags: flag_map,
            flag_set_metadata,
        })
    }

    fn transpose_evaluator_refs(configuration: &mut Value) -> Result<(), FlagdEvaluationError> {
        let evaluators = configuration
            .get("$evaluators")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();

        let Some(flags) = configuration
            .get_mut("flags")
            .and_then(Value::as_object_mut)
        else {
            return Ok(());
        };

        for flag in flags.values_mut() {
            if let Some(targeting) = flag.get_mut("targeting") {
                Self::resolve_refs(targeting, &evaluators, &mut Vec::new())?;
            }
        }

        Ok(())
    }

    fn resolve_refs(
        value: &mut Value,
        evaluators: &Map<String, Value>,
        stack: &mut Vec<String>,
    ) -> Result<(), FlagdEvaluationError> {
        match value {
            Value::Object(obj) => {
                if obj.len() == 1
                    && let Some(ref_name) = obj.get("$ref").and_then(Value::as_str)
                {
                    if stack.iter().any(|name| name == ref_name) {
                        return Err(FlagdEvaluationError::Parse(format!(
                            "Circular evaluator reference detected: {}",
                            ref_name
                        )));
                    }

                    let mut replacement = evaluators.get(ref_name).cloned().ok_or_else(|| {
                        FlagdEvaluationError::Parse(format!(
                            "Evaluator reference '{}' was not found",
                            ref_name
                        ))
                    })?;

                    stack.push(ref_name.to_string());
                    Self::resolve_refs(&mut replacement, evaluators, stack)?;
                    stack.pop();

                    *value = replacement;
                    return Ok(());
                }

                for child in obj.values_mut() {
                    Self::resolve_refs(child, evaluators, stack)?;
                }
                Ok(())
            }
            Value::Array(items) => {
                for item in items {
                    Self::resolve_refs(item, evaluators, stack)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn convert_map_to_hashmap(map: &Map<String, Value>) -> HashMap<String, serde_json::Value> {
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

#[cfg(test)]
#[path = "flag_parser_test.rs"]
mod tests;
