use datalogic_rs::bumpalo::Bump;
use datalogic_rs::operator::EvalContext;
use datalogic_rs::{ArenaExt, CustomOperator, DataValue};
use murmurhash3::murmurhash3_x86_32;
use tracing::debug;

pub struct FractionalOperator;

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
                if arr.len() >= 2 {
                    let variant = arr[0].as_str().unwrap_or("").to_string();
                    let weight = arr[1].as_i64().unwrap_or(1) as i32;

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
