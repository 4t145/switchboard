use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveryInfo {
    pub connection: DiscoveryConnectionInfo,
    pub kernel: crate::kernel::KernelInfo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveryConnectionInfo {
    pub grpc: String,
}
