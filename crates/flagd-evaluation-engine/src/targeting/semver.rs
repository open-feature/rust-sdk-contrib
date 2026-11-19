use datalogic_rs::bumpalo::Bump;
use datalogic_rs::operator::EvalContext;
use datalogic_rs::{ArenaExt, CustomOperator, DataValue};
use semver::{BuildMetadata, Version};
use tracing::debug;

pub struct SemVerOperator;

/// Parses a version leniently: an optional `v`/`V` prefix and missing minor/patch parts
/// (`1`, `1.0`) are accepted. Build metadata is dropped because it never affects precedence.
fn parse_version(input: &str) -> Option<Version> {
    let s = input.trim();
    let s = s.strip_prefix(['v', 'V']).unwrap_or(s);
    let (core, suffix) = s.split_at(s.find(['-', '+']).unwrap_or(s.len()));
    let padded = match core.matches('.').count() {
        0 => format!("{core}.0.0"),
        1 => format!("{core}.0"),
        _ => core.to_string(),
    };
    let mut version = Version::parse(&format!("{padded}{suffix}")).ok()?;
    version.build = BuildMetadata::EMPTY;
    Some(version)
}

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
            // Numeric context values (e.g. `1` or `1.2`) are treated as partial versions.
            if let Some(i) = arg.as_i64() {
                return Some(i.to_string());
            }
            if let Some(f) = arg.as_f64() {
                return Some(f.to_string());
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

        let Some(version1) = parse_version(&version1_str) else {
            debug!("Failed to parse first version: {:?}", version1_str);
            return Ok(arena.null());
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

        let Some(version2) = parse_version(&version2_str) else {
            debug!("Failed to parse second version: {:?}", version2_str);
            return Ok(arena.null());
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

#[cfg(test)]
mod tests {
    use super::parse_version;

    #[test]
    fn parse_version_accepts_lenient_forms() {
        for s in ["1.0.0", "v1.0.0", "V1.0.0", "1.0", "1", "1.0.0+build"] {
            assert_eq!(parse_version(s), parse_version("1.0.0"), "{s}");
        }
        assert!(parse_version("1.2.3-rc.1").unwrap() < parse_version("1.2.3").unwrap());
        assert!(parse_version("1.2").unwrap() > parse_version("1.1.9").unwrap());
        assert!(parse_version("not-a-version").is_none());
        assert!(parse_version("").is_none());
    }
}
