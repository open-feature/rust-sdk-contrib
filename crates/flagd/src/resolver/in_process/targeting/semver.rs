use anyhow::Result;
use semver::Version;
use tracing::debug;
use datalogic_rs::datalogic::CustomOperator;
use datalogic_rs::logic::error::LogicError;
use datalogic_rs::value::DataValue;

#[derive(Debug)]
pub struct SemVerOperator;

impl CustomOperator for SemVerOperator {
    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue, LogicError> {
        if args.len() != 3 {
            debug!("SemVer comparison requires exactly 3 arguments");
            return Err(LogicError::InvalidArgumentsError);
        }

        // Extract version 1
        let version1 = match &args[0] {
            DataValue::String(s) => {
                match Version::parse(s) {
                    Ok(v) => v,
                    Err(e) => {
                        debug!("Failed to parse first version '{}': {}", s, e);
                        return Ok(DataValue::Null);
                    }
                }
            },
            _ => {
                debug!("First argument must be a string representing a version");
                return Ok(DataValue::Null);
            }
        };

        // Extract operator
        let operator = match &args[1] {
            DataValue::String(op) => op,
            _ => {
                debug!("Second argument must be a string representing the comparison operator");
                return Ok(DataValue::Null);
            }
        };

        // Extract version 2
        let version2 = match &args[2] {
            DataValue::String(s) => {
                match Version::parse(s) {
                    Ok(v) => v,
                    Err(e) => {
                        debug!("Failed to parse second version '{}': {}", s, e);
                        return Ok(DataValue::Null);
                    }
                }
            },
            _ => {
                debug!("Third argument must be a string representing a version");
                return Ok(DataValue::Null);
            }
        };

        // Perform comparison
        let result = match *operator {
            "=" => version1 == version2,
            "!=" => version1 != version2,
            "<" => version1 < version2,
            "<=" => version1 <= version2,
            ">" => version1 > version2,
            ">=" => version1 >= version2,
            "^" => version1.major == version2.major,
            "~" => version1.major == version2.major && version1.minor == version2.minor,
            _ => {
                debug!("Unsupported operator: {}", operator);
                return Ok(DataValue::Null);
            }
        };

        debug!("SemVer comparison: {} {} {} = {}", version1, operator, version2, result);
        Ok(DataValue::Bool(result))
    }
}
