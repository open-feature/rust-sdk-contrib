use ciborium::Value as CborValue;
use datalogic_rs::bumpalo::Bump;
use datalogic_rs::operator::EvalContext;
use datalogic_rs::{ArenaExt, CustomOperator, DataValue};
use murmurhash3::murmurhash3_x86_32;
use tracing::debug;

pub struct FractionalOperator;

fn serialize_cbor(cbor_val: &CborValue) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    ciborium::into_writer(cbor_val, &mut bytes).ok()?;
    Some(bytes)
}

fn data_value_to_cbor(val: &DataValue) -> Option<CborValue> {
    match val {
        DataValue::Null => Some(CborValue::Null),
        DataValue::Bool(b) => Some(CborValue::Bool(*b)),
        DataValue::String(s) => Some(CborValue::Text((*s).to_string())),
        DataValue::Number(num) => {
            if let Some(i) = num.as_i64() {
                Some(CborValue::Integer(i.into()))
            } else {
                let f = num.as_f64();
                if !f.is_finite() {
                    return None;
                }
                Some(CborValue::Float(f))
            }
        }
        DataValue::Array(items) => {
            let mut cbor_items = Vec::with_capacity(items.len());
            for item in *items {
                cbor_items.push(data_value_to_cbor(item)?);
            }
            Some(CborValue::Array(cbor_items))
        }
        DataValue::Object(entries) => {
            let mut sorted_entries: Vec<_> = entries.iter().collect();
            sorted_entries.sort_by(|(k1, _), (k2, _)| {
                k1.len()
                    .cmp(&k2.len())
                    .then_with(|| k1.as_bytes().cmp(k2.as_bytes()))
            });
            let mut cbor_map = Vec::with_capacity(sorted_entries.len());
            for (k, v) in sorted_entries {
                cbor_map.push((CborValue::Text((*k).to_string()), data_value_to_cbor(v)?));
            }
            Some(CborValue::Map(cbor_map))
        }
    }
}

fn encode_data_value_cbor(val: &DataValue) -> Option<Vec<u8>> {
    let cbor_val = data_value_to_cbor(val)?;
    serialize_cbor(&cbor_val)
}

fn bucket_for(bucket_by_bytes: &[u8], total_weight: u64) -> (u32, u64) {
    let hash = murmurhash3_x86_32(bucket_by_bytes, 0);
    let bucket = ((hash as u64) * total_weight) >> 32;
    (hash, bucket)
}

impl CustomOperator for FractionalOperator {
    fn evaluate<'a>(
        &self,
        args: &[&'a DataValue<'a>],
        context: &mut EvalContext<'_, 'a>,
        arena: &'a Bump,
    ) -> std::result::Result<&'a DataValue<'a>, datalogic_rs::Error> {
        if args.is_empty() {
            debug!("No arguments provided for fractional targeting.");
            return Ok(arena.null());
        }

        // If the first element is a non-array, non-null value, use it as the bucketing expression and use remaining elements as buckets.
        // Otherwise, compute the bucketing key from provided data and treat the whole array as bucket definitions.
        let (bucket_by_bytes, distributions) = if args[0].is_null() {
            debug!("Fractional logic failed: randomization unit evaluated to null");
            return Ok(arena.null());
        } else if !args[0].is_array() {
            if args.len() < 2 {
                debug!(
                    "Fractional logic failed: insufficient arguments for bucketing expression and distributions"
                );
                return Ok(arena.null());
            }
            let Some(bytes) = encode_data_value_cbor(args[0]) else {
                debug!("Fractional logic failed: unable to encode bucketing expression to CBOR");
                return Ok(arena.null());
            };
            (bytes, &args[1..])
        } else {
            let data = context.root_input();

            let Some(tk_val) = data.get("targetingKey").filter(|v| !v.is_null()) else {
                debug!("Fractional logic failed: missing targetingKey in context");
                return Ok(arena.null());
            };
            let Some(targeting_key) = tk_val.as_str() else {
                debug!("Fractional logic failed: targetingKey in context is not a string");
                return Ok(arena.null());
            };
            if targeting_key.is_empty() {
                debug!("Fractional logic failed: targetingKey in context is empty");
                return Ok(arena.null());
            }
            let flag_key = data
                .get("$flagd")
                .and_then(|v| v.get("flagKey"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let fallback_array = CborValue::Array(vec![
                CborValue::Text(flag_key.to_string()),
                CborValue::Text(targeting_key.to_string()),
            ]);
            let Some(bytes) = serialize_cbor(&fallback_array) else {
                return Ok(arena.null());
            };
            (bytes, args)
        };

        let mut total_weight: u64 = 0;
        let mut buckets = Vec::with_capacity(distributions.len());

        for dist in distributions {
            let Some(arr) = dist.as_array() else {
                return Ok(arena.null());
            };
            if arr.is_empty() {
                return Ok(arena.null());
            }
            let variant = &arr[0];
            if variant.is_array() || variant.is_object() {
                return Ok(arena.null());
            }
            let weight: u64 = if arr.len() >= 2 {
                let Some(w) = arr[1].as_f64() else {
                    return Ok(arena.null());
                };
                if !w.is_finite() || w.fract() != 0.0 || w > (i32::MAX as f64) {
                    return Ok(arena.null());
                }
                let w = if w < 0.0 { 0.0 } else { w };
                w as u64
            } else {
                1
            };
            total_weight += weight;
            if total_weight > (i32::MAX as u64) {
                return Ok(arena.null());
            }
            buckets.push((variant, weight));
        }
        if total_weight == 0 {
            return Ok(arena.null());
        }
        let (hash, bucket) = bucket_for(&bucket_by_bytes, total_weight);
        debug!(
            "Fractional evaluation: bucket_by_bytes={:02x?}, hash={}, bucket={}",
            bucket_by_bytes, hash, bucket
        );
        let mut range_start: u64 = 0;
        for (variant, weight) in &buckets {
            let range_end = range_start + *weight;
            if bucket >= range_start && bucket < range_end {
                return Ok(*variant);
            }
            range_start = range_end;
        }
        Ok(arena.null())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::targeting::Operator;
    use open_feature::{EvaluationContext, EvaluationContextFieldValue, StructValue, Value};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn bucket_for_uses_unsigned_murmurhash_shift() {
        let (hash, bucket) = bucket_for(b"flag-user", 100);

        assert_eq!(hash, 2_410_693_464);
        assert_eq!(bucket, 56);
    }

    #[test]
    fn fractional_evaluates_non_string_primitives_and_float_normalization() {
        let operator = Operator::new();
        let rule = r#"{
            "fractional": [
                {"var": "hashing_input"},
                ["bucket1", 10],
                ["bucket2", 10],
                ["bucket3", 10],
                ["bucket4", 10],
                ["bucket5", 10],
                ["bucket6", 10],
                ["bucket7", 10],
                ["bucket8", 10],
                ["bucket9", 10],
                ["bucket10", 10]
            ]
        }"#;

        let ctx_int = EvaluationContext::default().with_custom_field("hashing_input", 1i64);
        let ctx_float = EvaluationContext::default().with_custom_field("hashing_input", 1.0f64);
        assert_eq!(
            operator.apply("test-flag", rule, &ctx_int).unwrap(),
            operator.apply("test-flag", rule, &ctx_float).unwrap()
        );

        let ctx_zero_int = EvaluationContext::default().with_custom_field("hashing_input", 0i64);
        let ctx_zero_float =
            EvaluationContext::default().with_custom_field("hashing_input", 0.0f64);
        let ctx_neg_zero_float =
            EvaluationContext::default().with_custom_field("hashing_input", -0.0f64);
        let res_zero = operator.apply("test-flag", rule, &ctx_zero_int).unwrap();
        assert_eq!(
            res_zero,
            operator.apply("test-flag", rule, &ctx_zero_float).unwrap()
        );
        assert_eq!(
            res_zero,
            operator
                .apply("test-flag", rule, &ctx_neg_zero_float)
                .unwrap()
        );
    }

    #[test]
    fn fractional_evaluates_objects_with_deterministic_key_ordering() {
        let operator = Operator::new();
        let rule = r#"{
            "fractional": [
                {"var": "hashing_input"},
                ["a", 25],
                ["b", 25],
                ["c", 25],
                ["d", 25]
            ]
        }"#;

        let mut fields1 = HashMap::new();
        fields1.insert("z".to_string(), Value::Int(1));
        fields1.insert("aa".to_string(), Value::Int(2));
        let ctx1 = EvaluationContext::default().with_custom_field(
            "hashing_input",
            EvaluationContextFieldValue::Struct(Arc::new(StructValue { fields: fields1 })),
        );

        let mut fields2 = HashMap::new();
        fields2.insert("aa".to_string(), Value::Int(2));
        fields2.insert("z".to_string(), Value::Int(1));
        let ctx2 = EvaluationContext::default().with_custom_field(
            "hashing_input",
            EvaluationContextFieldValue::Struct(Arc::new(StructValue { fields: fields2 })),
        );

        assert_eq!(
            operator.apply("test-flag", rule, &ctx1).unwrap(),
            operator.apply("test-flag", rule, &ctx2).unwrap()
        );
    }

    #[test]
    fn fractional_implicit_targeting_key_rejects_missing_or_empty() {
        let operator = Operator::new();
        let rule = r#"{
            "fractional": [
                ["heads", 50],
                ["tails", 50]
            ]
        }"#;

        let empty_ctx = EvaluationContext::default();
        assert_eq!(operator.apply("test-flag", rule, &empty_ctx).unwrap(), None);

        let blank_tk_ctx = EvaluationContext::default().with_targeting_key("");
        assert_eq!(
            operator.apply("test-flag", rule, &blank_tk_ctx).unwrap(),
            None
        );

        let valid_tk_ctx = EvaluationContext::default().with_targeting_key("user-1");
        assert_eq!(
            operator
                .apply("fractional-flag-shorthand", rule, &valid_tk_ctx)
                .unwrap(),
            Some("heads".to_string())
        );
    }
}
