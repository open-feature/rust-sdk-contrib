use datalogic_rs::DataValue;
use murmurhash3::murmurhash3_x86_32;
use tracing::debug;

#[derive(Debug)]
pub struct Fractional;

impl Fractional {
    // Simple operator function implementation that works with owned DataValues
    pub fn fractional_op(args: Vec<DataValue>) -> std::result::Result<DataValue, String> {
        if args.is_empty() {
            debug!("No arguments provided for fractional targeting.");
            return Ok(DataValue::Null);
        }

        // Get the targeting key and flag key from the data context
        // Note: in the simple operator pattern, context data is passed as the first argument
        let (bucket_by, distributions) = if args.len() > 1 && args[0].is_string() {
            // If the first element is a string, use it as the bucketing expression
            let bucket_key = args[0].as_str().unwrap_or_default().to_string();
            debug!("Using explicit bucketing expression: {:?}", bucket_key);
            (bucket_key, args[1..].to_vec())
        } else {
            // Otherwise, construct the bucket key from the current context
            // This requires accessing global context which isn't directly available in the simple operator
            // So we'll use a default approach here
            let computed = "default_key".to_string();
            debug!("Using default bucketing key: {:?}", computed);
            (computed, args)
        };

        if distributions.is_empty() {
            debug!("No bucket definitions provided.");
            return Ok(DataValue::Null);
        }

        let mut total_weight = 0;
        let mut buckets = Vec::new();
        for dist in &distributions {
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
                // In this simple operator pattern, we need to create an owned DataValue
                // Since variant is already a String (owned), we can use into_boxed_str to get a 'static str
                let boxed_str = variant.into_boxed_str();
                let static_str = Box::leak(boxed_str);
                return Ok(DataValue::String(static_str));
            }
        }

        debug!("No bucket matched for bucket value: {:.4}", bucket);
        Ok(DataValue::Null)
    }
}
