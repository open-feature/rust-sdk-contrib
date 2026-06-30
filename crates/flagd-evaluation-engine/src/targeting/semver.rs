use datalogic_rs::bumpalo::Bump;
use datalogic_rs::operator::EvalContext;
use datalogic_rs::{ArenaExt, CustomOperator, DataValue};
use semver::Version;
use tracing::debug;

pub struct SemVerOperator;

impl CustomOperator for SemVerOperator {
    fn evaluate<'a>(
        &self,
        args: &[&'a DataValue<'a>],
        context: &mut EvalContext<'_, 'a>,
        arena: &'a Bump,
    ) -> datalogic_rs::Result<&'a DataValue<'a>> {
        if args.len() != 3 {
            debug!("SemVer requires exactly 3 arguments, got {}", args.len());
            return Ok(arena.null());
        }

        // Helper function to resolve a value (either a string literal or a variable reference)
        let resolve_value = |arg: &DataValue<'a>| -> Option<String> {
            if let Some(s) = arg.as_str() {
                return Some(s.to_string());
            }

            // Custom operator arguments are pre-evaluated by datalogic-rs 5.x.
            // Keep this fallback for object literals that explicitly encode a var reference.
            let var_name = arg.get("var").and_then(|v| v.as_str())?;
            let data = context.root_input();
            data.get(var_name)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    let (prefix, field) = var_name.split_once('.')?;
                    (prefix == "$flagd").then(|| {
                        data.get("$flagd")
                            .and_then(|v| v.get(field))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })?
                })
        };

        let version1_str = match resolve_value(args[0]) {
            Some(s) => s,
            None => {
                debug!(
                    "First argument must be a string or variable reference: {:?}",
                    args[0]
                );
                return Ok(arena.null());
            }
        };

        let version1 = match Version::parse(&version1_str) {
            Ok(v) => v,
            Err(e) => {
                debug!("Failed to parse first version: {:?}: {}", version1_str, e);
                return Ok(arena.null());
            }
        };

        let operator = match args[1].as_str() {
            Some(s) => s,
            None => {
                debug!("Operator must be a string: {:?}", args[1]);
                return Ok(arena.null());
            }
        };

        let version2_str = match resolve_value(args[2]) {
            Some(s) => s,
            None => {
                debug!(
                    "Second argument must be a string or variable reference: {:?}",
                    args[2]
                );
                return Ok(arena.null());
            }
        };

        let version2 = match Version::parse(&version2_str) {
            Ok(v) => v,
            Err(e) => {
                debug!("Failed to parse second version: {:?}: {}", version2_str, e);
                return Ok(arena.null());
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
                return Ok(arena.null());
            }
        };

        debug!("SemVer comparison result: {}", result);
        Ok(arena.bool(result))
    }
}
