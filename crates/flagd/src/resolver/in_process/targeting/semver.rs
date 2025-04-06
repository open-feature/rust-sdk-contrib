use anyhow::Result;
use datalogic_rs::{arena::DataArena, DataValue};
use semver::Version;
use tracing::debug;

#[derive(Debug)]
pub struct SemVer;

impl datalogic_rs::arena::CustomOperator for SemVer {
    fn evaluate<'a>(
        &self,
        args: &'a [DataValue<'a>],
        arena: &'a DataArena,
    ) -> std::result::Result<&'a DataValue<'a>, datalogic_rs::logic::LogicError> {
        // Main implementation
        let result: Result<&'a DataValue<'a>> = (|| {
            if args.len() != 3 {
                debug!("SemVer requires exactly 3 arguments, got {}", args.len());
                return Ok(arena.null_value());
            }

            let version1 = match &args[0] {
                DataValue::String(s) => match Version::parse(s) {
                    Ok(v) => v,
                    Err(e) => {
                        debug!("Failed to parse first version: {:?}: {}", s, e);
                        return Ok(arena.null_value());
                    }
                },
                _ => {
                    debug!("First argument must be a string: {:?}", args[0]);
                    return Ok(arena.null_value());
                }
            };

            let operator = match &args[1] {
                DataValue::String(s) => s,
                _ => {
                    debug!("Operator must be a string: {:?}", args[1]);
                    return Ok(arena.null_value());
                }
            };

            let version2 = match &args[2] {
                DataValue::String(s) => match Version::parse(s) {
                    Ok(v) => v,
                    Err(e) => {
                        debug!("Failed to parse second version: {:?}: {}", s, e);
                        return Ok(arena.null_value());
                    }
                },
                _ => {
                    debug!("Second argument must be a string: {:?}", args[2]);
                    return Ok(arena.null_value());
                }
            };

            debug!("Comparing {} {} {}", version1, operator, version2);
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
                    debug!("Unknown operator: {}", operator);
                    return Ok(arena.null_value());
                }
            };

            debug!("SemVer comparison result: {}", result);
            if result {
                Ok(arena.true_value())
            } else {
                Ok(arena.false_value())
            }
        })();

        // Convert any anyhow error to the expected datalogic error type
        result.map_err(|_| datalogic_rs::logic::LogicError::OperatorNotFoundError {
            operator: "sem_ver".to_string(),
        })
    }
}
