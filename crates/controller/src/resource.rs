pub mod tls;
pub mod gateway;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct ResourceConfig {
    pub tls: tls::TlsResourceConfig,
    pub gateway: gateway::GatewayResourceConfig,
}