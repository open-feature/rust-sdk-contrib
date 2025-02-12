use anyhow::Result;
use datalogic_rs::{JsonLogic, Rule};
use open_feature::{EvaluationContext, EvaluationContextFieldValue};
use serde_json::{Map, Value, json};

mod fractional;
mod semver;
mod string_comp;

use fractional::Fractional;
use semver::SemVer;
use string_comp::{StringComp, StringCompType};

#[derive(Clone)]
pub struct Operator {}

impl Operator {
    pub fn new() -> Operator {
        Operator {}
    }

    pub fn apply(
        &self,
        flag_key: &str,
        targeting_rule: &str,
        ctx: &EvaluationContext,
    ) -> Result<Option<String>> {
        let rule_value: Value = serde_json::from_str(targeting_rule)?;
        let result = self.evaluate_rule(&rule_value, flag_key, ctx)?;
        Ok(result.as_str().map(String::from))
    }

    fn evaluate_rule(&self, rule: &Value, flag_key: &str, ctx: &EvaluationContext) -> Result<Value> {
        match rule {
            Value::Object(map) => {
                if let Some((op, args)) = map.iter().next() {
                    match op.as_str() {
                        "if" => {
                            if let Value::Array(conditions) = args {
                                if conditions.len() >= 2 {
                                    let condition = self.evaluate_rule(&conditions[0], flag_key, ctx)?;
                                    match condition {
                                        Value::Bool(true) => Ok(conditions[1].clone()),
                                        Value::Bool(false) if conditions.len() > 2 => Ok(conditions[2].clone()),
                                        _ => Ok(Value::Null)
                                    }
                                } else {
                                    Ok(Value::Null)
                                }
                            } else {
                                Ok(Value::Null)
                            }
                        },
                        "fractional" => {
                            if let Value::Array(args) = args {
                                let data = self.build_evaluation_data(flag_key, ctx);
                                let data_map = data.as_object()
                                    .map(|obj| obj.iter()
                                        .map(|(k, v)| (k.clone(), v.clone()))
                                        .collect())
                                    .unwrap_or_default();
                                Fractional::evaluate(args, &data_map)
                            } else {
                                Ok(Value::Null)
                            }
                        },
                        "ends_with" => {
                            if let Value::Array(args) = args {
                                let mut resolved_args = Vec::new();
                                for arg in args {
                                    match arg {
                                        Value::Object(_) => {
                                            resolved_args.push(self.evaluate_rule(arg, flag_key, ctx)?);
                                        },
                                        _ => resolved_args.push(arg.clone()),
                                    }
                                }
                                StringComp::evaluate(StringCompType::EndsWith, &resolved_args)
                            } else {
                                Ok(Value::Null)
                            }
                        },
                        "starts_with" => {
                            if let Value::Array(args) = args {
                                let mut resolved_args = Vec::new();
                                for arg in args {
                                    match arg {
                                        Value::Object(_) => {
                                            resolved_args.push(self.evaluate_rule(arg, flag_key, ctx)?);
                                        },
                                        _ => resolved_args.push(arg.clone()),
                                    }
                                }
                                StringComp::evaluate(StringCompType::StartsWith, &resolved_args)
                            } else {
                                Ok(Value::Null)
                            }
                        },
                        "sem_ver" => {
                            if let Value::Array(args) = args {
                                let mut resolved_args = Vec::new();
                                for arg in args {
                                    match arg {
                                        Value::Object(_) => {
                                            resolved_args.push(self.evaluate_rule(arg, flag_key, ctx)?);
                                        },
                                        _ => resolved_args.push(arg.clone()),
                                    }
                                }
                                SemVer::evaluate(&resolved_args)
                            } else {
                                Ok(Value::Null)
                            }
                        },
                        "var" => {
                            if let Some(path) = args.as_str() {
                                let data = self.build_evaluation_data(flag_key, ctx);
                                Ok(data.get(path).cloned().unwrap_or(Value::Null))
                            } else {
                                Ok(Value::Null)
                            }
                        },
                        _ => {
                            let rule = Rule::from_value(rule)?;
                            let data = self.build_evaluation_data(flag_key, ctx);
                            Ok(JsonLogic::apply(&rule, &data)?)
                        }
                    }
                } else {
                    Ok(rule.clone())
                }
            },
            _ => Ok(rule.clone())
        }
    }

    fn build_evaluation_data(&self, flag_key: &str, ctx: &EvaluationContext) -> Value {
        let mut data = Map::new();

        if let Some(targeting_key) = &ctx.targeting_key {
            data.insert(
                "targetingKey".to_string(), 
                Value::String(targeting_key.clone())
            );
        }

        let flagd_props = json!({
            "flagKey": flag_key,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        data.insert("$flagd".to_string(), flagd_props);

        for (key, value) in &ctx.custom_fields {
            data.insert(key.clone(), context_value_to_json(value));
        }

        Value::Object(data)
    }
}

fn context_value_to_json(value: &EvaluationContextFieldValue) -> Value {
    match value {
        EvaluationContextFieldValue::String(s) => Value::String(s.clone()),
        EvaluationContextFieldValue::Bool(b) => Value::Bool(*b),
        EvaluationContextFieldValue::Int(i) => Value::Number((*i).into()),
        EvaluationContextFieldValue::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                Value::Number(n)
            } else {
                Value::Null
            }
        }
        EvaluationContextFieldValue::DateTime(dt) => Value::String(dt.to_string()),
        EvaluationContextFieldValue::Struct(s) => Value::String(format!("{:?}", s)),
    }
}
