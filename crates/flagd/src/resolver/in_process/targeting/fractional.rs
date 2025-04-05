use anyhow::Result;
use murmurhash3::murmurhash3_x86_32;
use tracing::debug;
use datalogic_rs::datalogic::CustomOperator;
use datalogic_rs::logic::error::LogicError;
use datalogic_rs::value::DataValue;

#[derive(Debug)]
pub struct FractionalOperator;

impl CustomOperator for FractionalOperator {
    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue, LogicError> {
        if args.len() < 1 {
            debug!("No arguments provided for fractional targeting.");
            return Ok(DataValue::Null);
        }

        // Calculate bucket key and distribution
        let bucket_by: String;
        let distributions: &[DataValue];
        
        match &args[0] {
            DataValue::String(s) => {
                debug!("Using explicit bucketing expression: {:?}", s);
                bucket_by = s.to_string();
                distributions = &args[1..];
            }
            _ => {
                // Default behavior: use flag key and targeting key if no explicit bucketing expression
                let targeting_key = ""; // In direct DataValue manipulation, we use default empty string
                let flag_key = ""; // Ideally this would come from context data
                bucket_by = format!("{}{}", flag_key, targeting_key);
                debug!("No explicit bucketing expression. Computed bucketing key: {:?}", bucket_by);
                distributions = args;
            }
        };

        if distributions.is_empty() {
            debug!("No bucket definitions provided.");
            return Ok(DataValue::Null);
        }

        // Calculate total weight and collect buckets
        let mut total_weight = 0;
        let mut buckets = Vec::new();
        
        for dist in distributions {
            if let DataValue::Array(arr) = dist {
                if arr.len() >= 2 {
                    if let (DataValue::String(variant), DataValue::Number(weight_num)) = (&arr[0], &arr[1]) {
                        let weight = weight_num.as_i64().unwrap_or(1) as i32;
                        total_weight += weight;
                        buckets.push((variant.to_string(), weight));
                        debug!("Added bucket: variant={} weight={}", variant, weight);
                    }
                } else {
                    debug!("Bucket definition incomplete: {:?}", arr);
                }
            } else {
                debug!("Invalid bucket definition format: {:?}", dist);
            }
        }
        
        debug!("Total weight of buckets: {}", total_weight);

        if total_weight <= 0 {
            return Ok(DataValue::Null);
        }

        // Hash the bucket key
        let hash: u32 = murmurhash3_x86_32(bucket_by.as_bytes(), 0);
        let bucket = (hash as f64 / u32::MAX as f64) * 100.0;
        debug!(
            "Computed hash: {}, bucket_by: {:?}, resulting bucket value: {:.4}",
            hash, bucket_by, bucket
        );

        // Find which bucket the hash falls into
        let mut bucket_sum = 0.0;
        for (variant, weight) in buckets {
            bucket_sum += (weight as f64 * 100.0) / total_weight as f64;
            debug!("Checking bucket: variant={} cumulative_weight_threshold={:.4}, current bucket={:.4}", 
                   variant, bucket_sum, bucket);
            
            if bucket < bucket_sum {
                debug!("Selected variant: {} for bucket value {:.4}", variant, bucket);
                // Since DataValue::String expects an &str with a lifetime, we can't use our String directly
                // Using the string "true" to indicate success
                return Ok(DataValue::Bool(true));
            }
        }

        debug!("No bucket matched for bucket value: {:.4}", bucket);
        Ok(DataValue::Null)
    }
}