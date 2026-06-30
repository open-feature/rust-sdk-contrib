use datalogic_rs::bumpalo::Bump;
use datalogic_rs::operator::EvalContext;
use datalogic_rs::{ArenaExt, CustomOperator, DataValue};
use murmurhash3::murmurhash3_x86_32;
use tracing::debug;

pub struct FractionalOperator;

fn bucket_for(bucket_by: &str) -> (u32, f64) {
    let hash = murmurhash3_x86_32(bucket_by.as_bytes(), 0);
    let signed = hash as i32;
    let bucket = (signed as f64).abs() / (i32::MAX as f64) * 100.0;
    (hash, bucket)
}

impl CustomOperator for FractionalOperator {
    fn evaluate<'a>(
        &self,
        args: &[&'a DataValue<'a>],
        context: &mut EvalContext<'_, 'a>,
        arena: &'a Bump,
    ) -> datalogic_rs::Result<&'a DataValue<'a>> {
        if args.is_empty() {
            debug!("No arguments provided for fractional targeting.");
            return Ok(arena.null());
        }

        // Get the current data from context (root contains the data)
        let data = context.root_input();

        // If the first element is a simple string, use it as the bucketing expression and use remaining elements as buckets.
        // Otherwise, compute the bucketing key from provided data and treat the whole array as bucket definitions.
        let (bucket_by, distributions) = match args[0].as_str() {
            Some(s) => {
                debug!("Using explicit bucketing expression: {:?}", s);
                (s.to_string(), &args[1..])
            }
            None => {
                // Extract targeting key from context if available
                let targeting_key = data
                    .get("targetingKey")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Extract flag key from context if available
                let flag_key = data
                    .get("$flagd")
                    .and_then(|v| v.get("flagKey"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let computed = format!("{}{}", flag_key, targeting_key);
                debug!("Using computed bucketing key from context: {:?}", computed);
                (computed, args)
            }
        };

        if distributions.is_empty() {
            debug!("No bucket definitions provided.");
            return Ok(arena.null());
        }

        let mut total_weight = 0;
        let mut buckets = Vec::new();
        for dist in distributions {
            if let Some(arr) = dist.as_array() {
                if !arr.is_empty() {
                    let variant = arr[0].as_str().unwrap_or("").to_string();
                    let weight = arr.get(1).and_then(|value| value.as_i64()).unwrap_or(1) as i32;

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
        if total_weight <= 0 {
            debug!("No positive bucket weight provided.");
            return Ok(arena.null());
        }

        let (hash, bucket) = bucket_for(&bucket_by);
        debug!(
            "Computed hash: {}, bucket_by: {:?}, resulting bucket value: {:.4}",
            hash, bucket_by, bucket
        );

        let mut bucket_sum = 0.0;
        for (variant, weight) in buckets {
            bucket_sum += (weight as f64 * 100.0) / total_weight as f64;
            debug!(
                "Checking bucket: variant={} cumulative_weight_threshold={:.4}, current bucket={:.4}",
                variant, bucket_sum, bucket
            );
            if bucket < bucket_sum {
                debug!(
                    "Selected variant: {} for bucket value {:.4}",
                    variant, bucket
                );
                return Ok(arena.string(&variant));
            }
        }

        debug!("No bucket matched for bucket value: {:.4}", bucket);
        Ok(arena.null())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_for_uses_signed_murmurhash_ratio() {
        let (hash, bucket) = bucket_for("flag-user");

        assert_eq!(hash, 2_410_693_464);
        assert!((bucket - 87.74333786579935).abs() < 1e-9);
    }
}
