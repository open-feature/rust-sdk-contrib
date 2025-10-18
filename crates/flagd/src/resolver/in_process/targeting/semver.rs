use datalogic_rs::{ContextStack, Evaluator, Operator};
use semver::Version;
use serde_json::Value;
use tracing::debug;

pub struct SemVerOperator;

impl Operator for SemVerOperator {
    fn evaluate(
        &self,
        args: &[Value],
        _context: &mut ContextStack,
        _evaluator: &dyn Evaluator,
    ) -> datalogic_rs::Result<Value> {
        if args.len() != 3 {
            debug!("SemVer requires exactly 3 arguments, got {}", args.len());
            return Ok(Value::Null);
        }

        let version1 = match args[0].as_str() {
            Some(s) => match Version::parse(s) {
                Ok(v) => v,
                Err(e) => {
                    debug!("Failed to parse first version: {:?}: {}", s, e);
                    return Ok(Value::Null);
                }
            },
            None => {
                debug!("First argument must be a string: {:?}", args[0]);
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

        let version2 = match args[2].as_str() {
            Some(s) => match Version::parse(s) {
                Ok(v) => v,
                Err(e) => {
                    debug!("Failed to parse second version: {:?}: {}", s, e);
                    return Ok(Value::Null);
                }
            },
            None => {
                debug!("Second argument must be a string: {:?}", args[2]);
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
