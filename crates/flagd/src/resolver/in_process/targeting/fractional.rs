use anyhow::Result;
use murmurhash3::murmurhash3_x86_32;
use serde_json::Value;
use std::collections::HashMap;

pub struct Fractional;

impl Fractional {
    pub fn evaluate(args: &[Value], data: &HashMap<String, Value>) -> Result<Value> {
        if args.len() < 2 {
            return Ok(Value::Null);
        }

        let bucket_by = match &args[0] {
            Value::String(s) => s.clone(),
            _ => {
                // Get targeting key from properties
                let targeting_key = data
                    .get("targetingKey")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let flag_key = data
                    .get("$flagd")
                    .and_then(|v| v.as_object())
                    .and_then(|o| o.get("flagKey"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                format!("{}{}", flag_key, targeting_key)
            }
        };

        let distributions = &args[1..];
        let mut total_weight = 0;
        let mut properties = Vec::new();

        for dist in distributions {
            if let Value::Array(arr) = dist {
                if arr.len() >= 2 {
                    let variant = arr[0].as_str().unwrap_or_default().to_string();
                    let weight = arr[1].as_u64().unwrap_or(1) as i32;
                    total_weight += weight;
                    properties.push((variant, weight));
                }
            }
        }

        let hash = murmurhash3_x86_32(bucket_by.as_bytes(), 0) as f32;
        let bucket = (hash.abs() / std::i32::MAX as f32) * 100.0;

        let mut bucket_sum = 0.0;
        for (variant, weight) in properties {
            bucket_sum += (weight as f32 * 100.0) / total_weight as f32;
            if bucket < bucket_sum {
                return Ok(Value::String(variant));
            }
        }

        Ok(Value::Null)
    }
}
