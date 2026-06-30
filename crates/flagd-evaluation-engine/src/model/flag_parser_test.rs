use super::*;
use serde_json::json;

#[test]
fn parse_string_transposes_evaluator_refs() {
    let config = r#"{
        "$evaluators": {
            "emailSuffix": { "ends_with": [{ "var": "email" }, "@example.com"] }
        },
        "flags": {
            "my-flag": {
                "state": "ENABLED",
                "variants": {
                    "variant-a": "a",
                    "variant-b": "b"
                },
                "defaultVariant": "variant-b",
                "targeting": {
                    "if": [{ "$ref": "emailSuffix" }, "variant-a", "variant-b"]
                }
            }
        }
    }"#;

    let result = FlagParser::parse_string(config).unwrap();
    let flag = result.flags.get("my-flag").unwrap();

    assert_eq!(
        flag.targeting.as_ref().unwrap(),
        &json!({
            "if": [
                { "ends_with": [{ "var": "email" }, "@example.com"] },
                "variant-a",
                "variant-b"
            ]
        })
    );
}

#[test]
fn parse_string_rejects_missing_evaluator_refs() {
    let config = r#"{
        "flags": {
            "my-flag": {
                "state": "ENABLED",
                "variants": {
                    "variant-a": "a",
                    "variant-b": "b"
                },
                "defaultVariant": "variant-b",
                "targeting": {
                    "if": [{ "$ref": "emailSuffix" }, "variant-a", "variant-b"]
                }
            }
        }
    }"#;

    let result = FlagParser::parse_string(config);

    assert!(result.is_err());
}

#[test]
fn parse_string_only_transposes_refs_in_targeting_rules() {
    let config = r#"{
        "$evaluators": {
            "emailSuffix": { "ends_with": [{ "var": "email" }, "@example.com"] }
        },
        "flags": {
            "my-flag": {
                "state": "ENABLED",
                "variants": {
                    "variant-a": { "$ref": "external-id" },
                    "variant-b": "b"
                },
                "defaultVariant": "variant-b",
                "metadata": {
                    "owner": { "$ref": "external-owner" }
                },
                "targeting": {
                    "if": [{ "$ref": "emailSuffix" }, "variant-a", "variant-b"]
                }
            }
        },
        "metadata": {
            "source": { "$ref": "external-source" }
        }
    }"#;

    let result = FlagParser::parse_string(config).unwrap();
    let flag = result.flags.get("my-flag").unwrap();

    assert_eq!(flag.variants["variant-a"], json!({ "$ref": "external-id" }));
    assert_eq!(flag.metadata["owner"], json!({ "$ref": "external-owner" }));
    assert_eq!(
        result.flag_set_metadata["source"],
        json!({ "$ref": "external-source" })
    );
    assert_eq!(
        flag.targeting.as_ref().unwrap(),
        &json!({
            "if": [
                { "ends_with": [{ "var": "email" }, "@example.com"] },
                "variant-a",
                "variant-b"
            ]
        })
    );
}
