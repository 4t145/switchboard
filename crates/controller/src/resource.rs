pub mod gateway;
pub mod tls;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct ResourceConfig {
    // pub tls: tls::TlsResourceConfig,
    pub gateway: gateway::GatewayResourceConfig,
}
