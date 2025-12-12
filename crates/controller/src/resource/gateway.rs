pub mod k8s;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct GatewayResourceConfig {
    pub k8s: Option<k8s::K8sGatewayResourceConfig>,
}