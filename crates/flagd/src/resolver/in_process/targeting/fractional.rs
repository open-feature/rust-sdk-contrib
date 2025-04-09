use datalogic_rs::DataValue;
use murmurhash3::murmurhash3_x86_32;
use tracing::debug;

// Function implementation for the SimpleOperatorFn approach
pub fn fractional_fn<'r>(
    args: Vec<DataValue<'r>>,
    data: DataValue<'r>,
) -> std::result::Result<DataValue<'r>, String> {
    if args.is_empty() {
        debug!("No arguments provided for fractional targeting.");
        return Ok(DataValue::Null);
    }

    // If the first element is a simple string, use it as the bucketing expression and use remaining elements as buckets.
    // Otherwise, compute the bucketing key from provided data and treat the whole array as bucket definitions.
    let (bucket_by, distributions) = match &args[0] {
        DataValue::String(s) => {
            debug!("Using explicit bucketing expression: {:?}", s);
            (s.to_string(), args[1..].to_vec())
        }
        _ => {
            // Extract targeting key from context if available
            let targeting_key = match &data {
                DataValue::Object(obj) => match obj.iter().find(|(k, _)| *k == "targetingKey") {
                    Some((_, DataValue::String(s))) => s.to_string(),
                    _ => String::new(),
                },
                _ => String::new(),
            };

            // Extract flag key from context if available
            let flag_key = match &data {
                DataValue::Object(obj) => match obj.iter().find(|(k, _)| *k == "$flagd") {
                    Some((_, DataValue::Object(flagd_obj))) => {
                        match flagd_obj.iter().find(|(k, _)| *k == "flagKey") {
                            Some((_, DataValue::String(s))) => s.to_string(),
                            _ => String::new(),
                        }
                    }
                    _ => String::new(),
                },
                _ => String::new(),
            };

            let computed = format!("{}{}", flag_key, targeting_key);
            debug!("Using computed bucketing key from context: {:?}", computed);
            (computed, args)
        }
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
                    _ => String::new(),
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
        debug!(
            "Checking bucket: variant={} cumulative_weight_threshold={:.4}, current bucket={:.4}",
            variant, bucket_sum, bucket
        );
        if bucket < bucket_sum {
            debug!(
                "Selected variant: {} for bucket value {:.4}",
                variant, bucket
            );
            // To return a string from a function, we need to do something unsafe to get a 'static lifetime
            // We'll use Box::leak to create a string with 'static lifetime
            // This is safe because the DataLogic library will copy this value to the arena
            let leaked_str: &'r str = Box::leak(variant.into_boxed_str());
            return Ok(DataValue::String(leaked_str));
        }
    }

    debug!("No bucket matched for bucket value: {:.4}", bucket);
    Ok(DataValue::Null)
}
