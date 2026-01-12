use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::ErrorStack;

pub const UDS_DEFAULT_PATH: &str = "/var/run/switchboard/kernel/default.sock";
pub const UDS_DEFAULT_DIR: &str = "/var/run/switchboard/kernel/";
pub const HTTP_DEFAULT_PORT: u16 = 8056;

#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]

#[serde(default)]
pub struct KernelInfo {
    pub name: String,
    pub id: String,
    pub description: Option<String>,
    #[serde(default)]
    pub meta: KernelMeta,
}

impl Default for KernelInfo {
    fn default() -> Self {
        KernelInfo {
            name: "Switchboard Kernel".to_string(),
            id: "default".to_string(),
            description: None,
            meta: KernelMeta::default(),
        }
    }
}



#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]

#[serde(default)]
pub struct KernelMeta {
    pub version: String,
    pub build: String,
}

impl Default for KernelMeta {
    fn default() -> Self {
        KernelMeta {
            version: crate::MODEL_VERSION.to_string(),
            build: "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]

#[serde(tag = "kind", content = "data")]
pub enum KernelStateKind {
    WaitingConfig,
    Running {
        config_version: String,
    },
    Updating {
        original_config_version:String,
        new_config_version: String,
    },
    ShuttingDown,
    Stopped,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]

pub struct KernelState {
    #[serde(flatten)]
    pub kind: KernelStateKind,
    #[bincode(with_serde)]
    pub since: chrono::DateTime<Utc>,
}

impl KernelState {
    pub fn init() -> Self {
        KernelState {
            kind: KernelStateKind::WaitingConfig,
            since: Utc::now(),
        }
    }
    pub fn new(kind: KernelStateKind) -> Self {
        KernelState {
            kind,
            since: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]

pub struct KernelInfoAndState {
    pub info: KernelInfo,
    pub state: KernelState,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]

#[serde(tag = "connection", content = "state")]
pub enum KernelConnectionAndState {
    Connected(KernelInfoAndState),
    FetchError(ErrorStack),
    Disconnected,
}