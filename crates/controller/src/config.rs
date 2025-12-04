use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use switchboard_model::{bytes::Base64Bytes, controller::ControllerInfo};
#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq)]
pub struct ControllerConfig {
    pub info: ControllerInfo,
    pub kernel: KernelConfig,
}


#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct KernelConfig {
    pub discovery: KernelDiscoveryConfig,
    pub psk: Base64Bytes,
}
#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq)]
pub struct KernelDiscoveryConfig {
    pub uds: KernelDiscoveryUdsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq)]
pub struct KernelDiscoveryUdsConfig {
    pub dir: PathBuf,
    pub scan_interval_secs: u32,
    pub max_frame_size: u32,
}

impl Default for KernelConfig {
    fn default() -> Self {
        KernelConfig {
            discovery: KernelDiscoveryConfig {
                uds: KernelDiscoveryUdsConfig {
                    dir: PathBuf::from(switchboard_model::kernel::UDS_DEFAULT_DIR),
                    scan_interval_secs: 10,
                    max_frame_size: 1 << 22,
                },
            },
            psk: Base64Bytes(rand::random::<[u8; 32]>().to_vec()),
        }
    }
}
