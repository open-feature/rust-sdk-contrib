use anyhow::Result;
use semver::Version;
use serde_json::Value;

pub struct SemVer;

impl SemVer {
    pub fn evaluate(args: &[Value]) -> Result<Value> {
        if args.len() != 3 {
            return Ok(Value::Null);
        }

        let version1 = match args[0].as_str().and_then(|s| Version::parse(s).ok()) {
            Some(v) => v,
            None => return Ok(Value::Null),
        };

        let operator = match args[1].as_str() {
            Some(op) => op,
            None => return Ok(Value::Null),
        };

        let version2 = match args[2].as_str().and_then(|s| Version::parse(s).ok()) {
            Some(v) => v,
            None => return Ok(Value::Null),
        };

        let result = match operator {
            "=" => version1 == version2,
            "!=" => version1 != version2,
            "<" => version1 < version2,
            "<=" => version1 <= version2,
            ">" => version1 > version2,
            ">=" => version1 >= version2,
            "^" => version1.major == version2.major,
            "~" => version1.major == version2.major && version1.minor == version2.minor,
            _ => return Ok(Value::Null),
        };

        Ok(Value::Bool(result))
    }
}
