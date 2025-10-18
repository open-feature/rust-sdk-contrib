use super::feature_flag::FeatureFlag;
use super::feature_flag::ParsingResult;
use anyhow::Result;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct FlagParser;

impl FlagParser {
    pub fn parse_string(configuration: &str) -> Result<ParsingResult> {
        let value: Value = serde_json::from_str(configuration)?;
        let obj = value
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid JSON structure"))?;

        let flags = obj
            .get("flags")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow::anyhow!("No flag configurations found in the payload"))?;

        let flag_set_metadata = obj
            .get("metadata")
            .and_then(|v| v.as_object())
            .map(Self::convert_map_to_hashmap)
            .unwrap_or_default();

        let mut flag_map = HashMap::new();
        for (key, value) in flags {
            let flag: FeatureFlag = serde_json::from_value(value.clone())?;
            flag_map.insert(key.clone(), flag);
        }

        Ok(ParsingResult {
            flags: flag_map,
            flag_set_metadata,
        })
    }

    fn convert_map_to_hashmap(map: &Map<String, Value>) -> HashMap<String, serde_json::Value> {
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}
