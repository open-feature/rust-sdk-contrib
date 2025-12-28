use flagd_evaluator::evaluation::{
    ErrorCode as EvaluatorErrorCode, EvaluationResult, ResolutionReason as EvaluatorReason,
};
use flagd_evaluator::model::FeatureFlag;
use open_feature::{
    EvaluationContext, EvaluationError, EvaluationErrorCode, EvaluationReason, FlagMetadata,
    FlagMetadataValue, StructValue, Value,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Helper to create an empty FeatureFlag for a given key when one doesn't exist
pub fn empty_flag(key: &str) -> FeatureFlag {
    FeatureFlag {
        key: Some(key.to_string()),
        state: "DISABLED".to_string(),
        default_variant: None,
        variants: Default::default(),
        targeting: None,
        metadata: Default::default(),
    }
}

/// Convert EvaluationContextFieldValue to JsonValue recursively
fn context_field_to_json(value: &open_feature::EvaluationContextFieldValue) -> JsonValue {
    use open_feature::EvaluationContextFieldValue;
    match value {
        EvaluationContextFieldValue::String(s) => JsonValue::String(s.clone()),
        EvaluationContextFieldValue::Bool(b) => JsonValue::Bool(*b),
        EvaluationContextFieldValue::Int(i) => JsonValue::Number((*i).into()),
        EvaluationContextFieldValue::Float(f) => {
            JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| {
                serde_json::Number::from_f64(0.0).unwrap()
            }))
        }
        EvaluationContextFieldValue::DateTime(dt) => JsonValue::String(dt.to_string()),
        EvaluationContextFieldValue::Struct(_) => {
            // NOTE: The OpenFeature Rust SDK stores structs as Arc<dyn Any> which cannot be
            // introspected or serialized. This is a known limitation - see the TODO comment in
            // the SDK source. Until this is fixed, we return an empty object to avoid breaking
            // targeting rules that expect an object type. This means nested struct fields in
            // evaluation context cannot be accessed by targeting rules.
            // See: https://github.com/open-feature/rust-sdk/blob/main/open-feature/src/evaluation/context_field_value.rs
            JsonValue::Object(serde_json::Map::new())
        }
    }
}

/// Build context JSON for evaluator from OpenFeature context
pub fn build_context_json(context: &EvaluationContext) -> JsonValue {
    let mut root = serde_json::Map::new();

    // Add targeting key if present
    if let Some(targeting_key) = &context.targeting_key {
        root.insert(
            "targetingKey".to_string(),
            JsonValue::String(targeting_key.clone()),
        );
    }

    // Add custom fields
    for (key, value) in &context.custom_fields {
        root.insert(key.clone(), context_field_to_json(value));
    }

    JsonValue::Object(root)
}

/// Map evaluator reason to OpenFeature reason
pub fn map_reason(reason: &EvaluatorReason) -> Option<EvaluationReason> {
    match reason {
        EvaluatorReason::Static => Some(EvaluationReason::Static),
        EvaluatorReason::Default => Some(EvaluationReason::Default),
        EvaluatorReason::TargetingMatch => Some(EvaluationReason::TargetingMatch),
        EvaluatorReason::Disabled => Some(EvaluationReason::Disabled),
        EvaluatorReason::Error
        | EvaluatorReason::FlagNotFound
        | EvaluatorReason::Fallback => Some(EvaluationReason::Error),
    }
}

/// Map evaluator error code to OpenFeature error code
pub fn map_error_code(code: &EvaluatorErrorCode) -> EvaluationErrorCode {
    match code {
        EvaluatorErrorCode::FlagNotFound => EvaluationErrorCode::FlagNotFound,
        EvaluatorErrorCode::ParseError => EvaluationErrorCode::ParseError,
        EvaluatorErrorCode::TypeMismatch => EvaluationErrorCode::TypeMismatch,
        EvaluatorErrorCode::General => {
            EvaluationErrorCode::General("Evaluation error".to_string())
        }
    }
}

/// Convert evaluation result to resolution details
pub fn result_to_details<T>(
    result: &EvaluationResult,
    value_extractor: impl Fn(&JsonValue) -> Option<T>,
) -> Result<open_feature::provider::ResolutionDetails<T>, EvaluationError> {
    use open_feature::provider::ResolutionDetails;

    // Check for errors
    if let Some(error_code) = &result.error_code {
        return Err(EvaluationError::builder()
            .code(map_error_code(error_code))
            .message(result.error_message.clone().unwrap_or_default())
            .build());
    }

    // Extract value
    let value = value_extractor(&result.value).ok_or_else(|| {
        EvaluationError::builder()
            .code(EvaluationErrorCode::TypeMismatch)
            .message("Value type mismatch".to_string())
            .build()
    })?;

    Ok(ResolutionDetails {
        value,
        variant: result.variant.clone(),
        reason: map_reason(&result.reason),
        flag_metadata: result.flag_metadata.as_ref().map(|metadata| {
            let mut flag_metadata = FlagMetadata::default();
            for (key, value) in metadata {
                if let Some(metadata_value) = json_to_metadata_value(value) {
                    flag_metadata = flag_metadata.with_value(key.clone(), metadata_value);
                }
            }
            flag_metadata
        }),
    })
}

/// Convert JsonValue to OpenFeature Value
pub fn json_to_value(v: &JsonValue) -> Value {
    match v {
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap())
            }
        }
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Object(obj) => {
            let fields = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::Struct(StructValue { fields })
        }
        JsonValue::Array(arr) => Value::Array(arr.iter().map(json_to_value).collect()),
        JsonValue::Null => Value::String(String::new()), // Default for null
    }
}

/// Convert JsonValue to FlagMetadataValue
pub fn json_to_metadata_value(v: &JsonValue) -> Option<FlagMetadataValue> {
    match v {
        JsonValue::String(s) => Some(FlagMetadataValue::String(s.clone())),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(FlagMetadataValue::Int(i))
            } else {
                n.as_f64().map(FlagMetadataValue::Float)
            }
        }
        JsonValue::Bool(b) => Some(FlagMetadataValue::Bool(*b)),
        _ => None, // FlagMetadata only supports primitives
    }
}

/// Get flag and metadata from evaluator storage
pub fn get_flag_and_metadata(
    flag_key: &str,
) -> (FeatureFlag, HashMap<String, serde_json::Value>) {
    let state = flagd_evaluator::storage::get_flag_state();
    let flag = state
        .as_ref()
        .and_then(|s| s.flags.get(flag_key))
        .cloned()
        .unwrap_or_else(|| empty_flag(flag_key));
    let metadata = state
        .as_ref()
        .map(|s| &s.flag_set_metadata)
        .cloned()
        .unwrap_or_default();
    (flag, metadata)
}
