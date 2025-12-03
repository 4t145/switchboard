use chrono::Utc;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub enum KernelStateKind {
    WaitingConfig,
    Running {
        config_signature: Vec<u8>,
    },
    Updating {
        original_config_signature: Vec<u8>,
        new_config_signature: Vec<u8>,
    },
    ShuttingDown,
    Stopped,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub struct KernelState {
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
}