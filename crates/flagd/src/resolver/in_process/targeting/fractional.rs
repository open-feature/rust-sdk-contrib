use anyhow::Result;
use datalogic_rs::{arena::DataArena, DataValue};
use murmurhash3::murmurhash3_x86_32;
use tracing::debug;

#[derive(Debug)]
pub struct Fractional;

impl datalogic_rs::arena::CustomOperator for Fractional {
    fn evaluate<'a>(
        &self,
        args: &'a [DataValue<'a>],
        arena: &'a DataArena,
    ) -> std::result::Result<&'a DataValue<'a>, datalogic_rs::logic::LogicError> {
        // Get the current data context
        let data = match arena.current_context(0) {
            Some(ctx) => ctx,
            None => arena.null_value(),
        };

        // Main implementation
        let result: Result<&'a DataValue<'a>> = (|| {
            if args.is_empty() {
                debug!("No arguments provided for fractional targeting.");
                return Ok(arena.null_value());
            }

            // If the first element is a simple string, use it as the bucketing expression and use remaining elements as buckets.
            // Otherwise, compute the bucketing key from provided data and treat the whole array as bucket definitions.
            let (bucket_by, distributions) = match &args[0] {
                DataValue::String(s) => {
                    debug!("Using explicit bucketing expression: {:?}", s);
                    (s.to_string(), &args[1..])
                }
                _ => {
                    let targeting_key = match data {
                        DataValue::Object(entries) => {
                            let mut key = "";
                            for (k, v) in *entries {
                                if *k == "targetingKey" {
                                    if let DataValue::String(s) = v {
                                        key = s;
                                        break;
                                    }
                                }
                            }
                            key
                        }
                        _ => "",
                    };

                    let flag_key = match data {
                        DataValue::Object(entries) => {
                            let mut key = "";
                            for (k, v) in *entries {
                                if *k == "$flagd" {
                                    if let DataValue::Object(flagd_entries) = v {
                                        for (fk, fv) in *flagd_entries {
                                            if *fk == "flagKey" {
                                                if let DataValue::String(s) = fv {
                                                    key = s;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                            key
                        }
                        _ => "",
                    };

                    let computed = format!("{}{}", flag_key, targeting_key);
                    debug!(
                        "No explicit bucketing expression. Computed bucketing key: {:?}",
                        computed
                    );
                    (computed, args)
                }
            };

            if distributions.is_empty() {
                debug!("No bucket definitions provided.");
                return Ok(arena.null_value());
            }

            let mut total_weight = 0;
            let mut buckets = Vec::new();
            for dist in distributions {
                if let DataValue::Array(arr) = dist {
                    if arr.len() >= 2 {
                        let variant = match &arr[0] {
                            DataValue::String(s) => s.to_string(),
                            _ => "".to_string(),
                        };

                        let weight = match &arr[1] {
                            DataValue::Number(n) => {
                                if let Some(i) = n.as_i64() {
                                    i as i32
                                } else {
                                    1
                                }
                            }
                            _ => 1,
                        };

                        total_weight += weight;
                        buckets.push((variant.clone(), weight));
                        debug!("Added bucket: variant={} weight={}", variant, weight);
                    } else {
                        debug!("Bucket definition incomplete: {:?}", arr);
                    }
                } else {
                    debug!("Invalid bucket definition format: {:?}", dist);
                }
            }
            debug!("Total weight of buckets: {}", total_weight);

            let hash: u32 = murmurhash3_x86_32(bucket_by.as_bytes(), 0);
            let bucket = (hash as f64 / u32::MAX as f64) * 100.0;
            debug!(
                "Computed hash: {}, bucket_by: {:?}, resulting bucket value: {:.4}",
                hash, bucket_by, bucket
            );

            let mut bucket_sum = 0.0;
            for (variant, weight) in buckets {
                bucket_sum += (weight as f64 * 100.0) / total_weight as f64;
                debug!("Checking bucket: variant={} cumulative_weight_threshold={:.4}, current bucket={:.4}", variant, bucket_sum, bucket);
                if bucket < bucket_sum {
                    debug!(
                        "Selected variant: {} for bucket value {:.4}",
                        variant, bucket
                    );
                    let string_value = arena.intern_str(&variant);
                    return Ok(arena.alloc(DataValue::String(string_value)));
                }
            }

            debug!("No bucket matched for bucket value: {:.4}", bucket);
            Ok(arena.null_value())
        })();

        // Convert any anyhow error to the expected datalogic error type
        result.map_err(|_| datalogic_rs::logic::LogicError::OperatorNotFoundError {
            operator: "fractional".to_string(),
        })
    }
}
