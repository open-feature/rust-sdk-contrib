use open_feature::Value;

pub trait ValueConverter {
    fn to_serde_json(&self) -> serde_json::Value;
    fn from_serde_json(value: &serde_json::Value) -> Option<Value>;
}

impl ValueConverter for Value {
    fn to_serde_json(&self) -> serde_json::Value {
        match self {
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(i) => serde_json::Value::Number((*i).into()),
            Value::Float(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Value::Array(_) => serde_json::Value::Array(vec![]),
            Value::Struct(_) => serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    fn from_serde_json(value: &serde_json::Value) -> Option<Value> {
        match value {
            serde_json::Value::String(s) => Some(Value::String(s.clone())),
            serde_json::Value::Bool(b) => Some(Value::Bool(*b)),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    Some(Value::Int(n.as_i64()?))
                } else {
                    Some(Value::Float(n.as_f64()?))
                }
            }
            _ => None,
        }
    }
}
