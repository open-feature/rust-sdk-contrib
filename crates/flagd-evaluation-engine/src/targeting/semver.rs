use datalogic_rs::{ContextStack, Evaluator, Operator};
use semver::Version;
use serde_json::Value;
use tracing::debug;

pub struct SemVerOperator;

impl Operator for SemVerOperator {
    fn evaluate(
        &self,
        args: &[Value],
        context: &mut ContextStack,
        _evaluator: &dyn Evaluator,
    ) -> datalogic_rs::Result<Value> {
        if args.len() != 3 {
            debug!("SemVer requires exactly 3 arguments, got {}", args.len());
            return Ok(Value::Null);
        }

        // Helper function to resolve a value (either a string literal or a variable reference)
        let resolve_value = |arg: &Value| -> Option<String> {
            match arg {
                Value::String(s) => Some(s.clone()),
                Value::Object(obj) => {
                    // Check if it's a variable reference: {"var": "variable_name"}
                    if let Some(var_name) = obj.get("var").and_then(|v| v.as_str()) {
                        // Get the current data from context
                        let frame = context.root();
                        let data = frame.data();
                        // Try to resolve from context (check both direct key and nested in $flagd)
                        data.get(var_name)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| {
                                // Also check in $flagd.flagKey format
                                if let Some(parts) = var_name.split_once('.') {
                                    if parts.0 == "$flagd" {
                                        data.get("$flagd")
                                            .and_then(|v| v.get(parts.1))
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        };

        let version1_str = match resolve_value(&args[0]) {
            Some(s) => s,
            None => {
                debug!(
                    "First argument must be a string or variable reference: {:?}",
                    args[0]
                );
                return Ok(Value::Null);
            }
        };

        let version1 = match Version::parse(&version1_str) {
            Ok(v) => v,
            Err(e) => {
                debug!("Failed to parse first version: {:?}: {}", version1_str, e);
                return Ok(Value::Null);
            }
        };

        let operator = match args[1].as_str() {
            Some(s) => s,
            None => {
                debug!("Operator must be a string: {:?}", args[1]);
                return Ok(Value::Null);
            }
        };

        let version2_str = match resolve_value(&args[2]) {
            Some(s) => s,
            None => {
                debug!(
                    "Second argument must be a string or variable reference: {:?}",
                    args[2]
                );
                return Ok(Value::Null);
            }
        };

        let version2 = match Version::parse(&version2_str) {
            Ok(v) => v,
            Err(e) => {
                debug!("Failed to parse second version: {:?}: {}", version2_str, e);
                return Ok(Value::Null);
            }
        };

        debug!("Comparing {} {} {}", version1, operator, version2);
        let result = match operator {
            "=" => version1 == version2,
            "!=" => version1 != version2,
            "<" => version1 < version2,
            "<=" => version1 <= version2,
            ">" => version1 > version2,
            ">=" => version1 >= version2,
            "^" => version1.major == version2.major,
            "~" => version1.major == version2.major && version1.minor == version2.minor,
            _ => {
                debug!("Unknown operator: {}", operator);
                return Ok(Value::Null);
            }
        };

        debug!("SemVer comparison result: {}", result);
        Ok(Value::Bool(result))
    }
}
