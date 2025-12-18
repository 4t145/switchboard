use serde::{Deserialize, Serialize};
use switchboard_custom_config::CustomConfig;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
#[derive(bon::Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct TcpServiceConfig<Cfg = CustomConfig> {
    pub provider: String,
    pub name: String,
    pub config: Option<Cfg>,
    pub description: Option<String>,
}

pub type TcpServiceConfigWithLink = TcpServiceConfig<switchboard_custom_config::Link>;

impl TcpServiceConfigWithLink {
    pub async fn resolve_links<R>(self, resolver: &R) -> Result<TcpServiceConfig, R::Error>
    where
        R: switchboard_custom_config::LinkResolver,
    {
        if let Some(linked_config) = self.config {
            let resolved_config = resolver.fetch(&linked_config).await?;
            Ok(TcpServiceConfig {
                provider: self.provider,
                name: self.name,
                config: Some(resolved_config),
                description: self.description,
            })
        } else {
            
            Ok(TcpServiceConfig {
                provider: self.provider,
                name: self.name,
                config: None,
                description: self.description,
            })
        }
    }
}