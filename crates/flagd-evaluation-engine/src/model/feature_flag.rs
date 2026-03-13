use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlag {
    pub state: String,
    pub default_variant: String,
    pub variants: HashMap<String, serde_json::Value>,
    pub targeting: Option<serde_json::Value>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl FeatureFlag {
    pub fn get_targeting(&self) -> String {
        self.targeting
            .as_ref()
            .map(|t| t.to_string())
            .unwrap_or_else(|| "{}".to_string())
    }
}

#[derive(Debug)]
pub struct ParsingResult {
    pub flags: HashMap<String, FeatureFlag>,
    pub flag_set_metadata: HashMap<String, serde_json::Value>,
}
