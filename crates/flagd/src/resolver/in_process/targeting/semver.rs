use datalogic_rs::DataValue;
use semver::Version;
use tracing::debug;

#[derive(Debug)]
pub struct SemVer;

impl SemVer {
    // Simple operator function implementation that works with owned DataValues
    pub fn semver_op(args: Vec<DataValue>) -> std::result::Result<DataValue, String> {
        if args.len() != 3 {
            debug!("SemVer requires exactly 3 arguments, got {}", args.len());
            return Ok(DataValue::Null);
        }

        let version1 = match &args[0] {
            DataValue::String(s) => match Version::parse(s) {
                Ok(v) => v,
                Err(e) => {
                    debug!("Failed to parse first version: {:?}: {}", s, e);
                    return Ok(DataValue::Null);
                }
            },
            _ => {
                debug!("First argument must be a string: {:?}", args[0]);
                return Ok(DataValue::Null);
            }
        };

        let operator = match &args[1] {
            DataValue::String(s) => s,
            _ => {
                debug!("Operator must be a string: {:?}", args[1]);
                return Ok(DataValue::Null);
            }
        };

        let version2 = match &args[2] {
            DataValue::String(s) => match Version::parse(s) {
                Ok(v) => v,
                Err(e) => {
                    debug!("Failed to parse second version: {:?}: {}", s, e);
                    return Ok(DataValue::Null);
                }
            },
            _ => {
                debug!("Second argument must be a string: {:?}", args[2]);
                return Ok(DataValue::Null);
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
                return Ok(DataValue::Null);
            }
        };

        debug!("SemVer comparison result: {}", result);
        Ok(DataValue::Bool(result))
    }
}
