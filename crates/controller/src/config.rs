use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use switchboard_model::{bytes::Base64Bytes, controller::ControllerInfo, protocol::DEFAULT_HEARTBEAT_INTERVAL_SECS};

use crate::{interface::InterfaceConfig, resource::ResourceConfig};







/// Controller Configuration
/// # Example
/// ```toml
/// [info]
/// name = "My Controller"
/// description = "This is my controller"
/// [kernel]
/// [kernel.discovery]
/// [kernel.discovery.uds]
/// dir = "/var/run/switchboard/kernels"
/// scan_interval_secs = 10
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq)]
pub struct ControllerConfig {
    #[serde(default)]
    pub info: ControllerInfo,
    #[serde(default)]
    pub kernel: KernelConfig,
    pub interface: InterfaceConfig,
    pub resource_config: ResourceConfig,
}


#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct KernelConfig {
    #[serde(default)]
    pub discovery: KernelDiscoveryConfig,
    #[serde(default)]
    pub connect: KernelConnectConfig,
    pub psk: Base64Bytes,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct KernelDiscoveryConfig {
    pub uds: KernelDiscoveryUdsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct KernelConnectConfig {
    pub heartbeat_interval: u32,
    pub channel_buffer_size: u32,
}

impl Default for KernelConnectConfig {
    fn default() -> Self {
        KernelConnectConfig {
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL_SECS,
            channel_buffer_size: 32,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct KernelDiscoveryUdsConfig {
    pub dir: PathBuf,
    pub scan_interval_secs: u32,
    pub max_frame_size: u32,
}

impl Default for KernelConfig {
    fn default() -> Self {
        KernelConfig {
            discovery: KernelDiscoveryConfig::default(),
            connect: KernelConnectConfig::default(),
            psk: Base64Bytes(rand::random::<[u8; 32]>().to_vec()),
        }
    }
}

impl Default for KernelDiscoveryUdsConfig {
    fn default() -> Self {
        KernelDiscoveryUdsConfig {
            dir: PathBuf::from(switchboard_model::kernel::UDS_DEFAULT_DIR),
            scan_interval_secs: 10,
            max_frame_size: 1 << 22,
        }
    }
}
