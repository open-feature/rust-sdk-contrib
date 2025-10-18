use flipt::error::UpstreamError;
use open_feature::{EvaluationContext, EvaluationError, EvaluationErrorCode, StructValue, Value};
use std::collections::HashMap;

pub(crate) fn translate_error(e: UpstreamError) -> EvaluationError {
    EvaluationError {
        code: EvaluationErrorCode::General(format!(
            "Flipt error: {}, message: \"{}\"",
            e.code, e.message
        )),
        message: Some(format!("{}", e)),
    }
}

pub(crate) fn translate_context(ctx: &EvaluationContext) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for (k, v) in ctx.custom_fields.iter() {
        if let Some(v) = v.as_str() {
            map.insert(k.clone(), v.to_owned());
        };
    }
    map
}

pub(crate) fn parse_json(json: &str) -> Result<Value, EvaluationError> {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            return Err(EvaluationError {
                code: EvaluationErrorCode::General("Parse error in JSON".to_owned()),
                message: Some(format!("Failed to parse JSON: {}", e)),
            });
        }
    };
    serde_to_openfeature_value(v)
}

pub(crate) fn serde_to_openfeature_value(v: serde_json::Value) -> Result<Value, EvaluationError> {
    match v {
        serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
        serde_json::Value::Number(n) => {
            let opt = if n.is_i64() {
                n.as_i64().map(Value::Int)
            } else if n.is_f64() {
                n.as_f64().map(Value::Float)
            } else {
                None
            };
            opt.map(Ok).unwrap_or(Err(EvaluationError {
                code: EvaluationErrorCode::General("Parse error in JSON".to_owned()),
                message: Some(format!(
                    "Expected a number of type i64 or f64, but found `{}`",
                    n
                )),
            }))
        }
        serde_json::Value::String(s) => Ok(Value::String(s)),
        serde_json::Value::Null => Err(EvaluationError {
            code: EvaluationErrorCode::General("Parse error in JSON".to_owned()),
            message: Some(format!("Unsupported JSON value: {}", v)),
        }),
        serde_json::Value::Array(a) => {
            let mut arr = Vec::new();
            for v in a {
                arr.push(serde_to_openfeature_value(v)?);
            }
            Ok(Value::Array(arr))
        }
        serde_json::Value::Object(o) => {
            let mut map = HashMap::new();
            for (k, v) in o {
                map.insert(k, serde_to_openfeature_value(v)?);
            }
            Ok(Value::Struct(StructValue { fields: map }))
        }
    }
}
