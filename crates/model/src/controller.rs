use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]

#[serde(default)]
pub struct ControllerInfo {
    pub name: String,
    pub description: Option<String>,
    pub meta: ControllerMeta,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]

#[serde(default)]
pub struct ControllerMeta {
    pub version: String,
    pub build: String,
}

impl Default for ControllerInfo {
    fn default() -> Self {
        ControllerInfo {
            name: "Switchboard Controller".to_string(),
            description: None,
            meta: ControllerMeta::default(),
        }
    }
}

impl Default for ControllerMeta {
    fn default() -> Self {
        ControllerMeta {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build: "unknown".to_string(),
        }
    }
}