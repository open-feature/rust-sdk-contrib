use anyhow::Result;
use serde_json::Value;
use tracing::debug;

pub enum StringCompType {
    StartsWith,
    EndsWith,
}

pub struct StringComp;

impl StringComp {
    pub fn evaluate(comp_type: StringCompType, args: &[Value]) -> Result<Value> {
        debug!("StringComp evaluating with args: {:?}", args);

        if args.len() != 2 {
            debug!("Invalid number of arguments: {}", args.len());
            return Ok(Value::Null);
        }

        let arg1 = match args[0].as_str() {
            Some(s) => {
                debug!("First argument: {}", s);
                s
            }
            None => {
                debug!("First argument is not a string: {:?}", args[0]);
                return Ok(Value::Null);
            }
        };

        let arg2 = match args[1].as_str() {
            Some(s) => {
                debug!("Second argument: {}", s);
                s
            }
            None => {
                debug!("Second argument is not a string: {:?}", args[1]);
                return Ok(Value::Null);
            }
        };

        let result = match comp_type {
            StringCompType::StartsWith => {
                debug!("Evaluating StartsWith: {} starts with {}", arg1, arg2);
                arg1.starts_with(arg2)
            }
            StringCompType::EndsWith => {
                debug!("Evaluating EndsWith: {} ends with {}", arg1, arg2);
                arg1.ends_with(arg2)
            }
        };

        debug!("String comparison result: {}", result);
        Ok(Value::Bool(result))
    }
}
