use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct K8sGatewayResourceConfig {
    pub gateway_namespace: String,
    
}