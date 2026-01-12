use serde::{Deserialize, Serialize};
use switchboard_custom_config::SerdeValue;

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Hash,
    bincode::Encode,
    bincode::Decode,
    PartialEq,
    Eq,
    bon::Builder,
)]

#[builder(on(String, into))]
pub struct TcpServiceConfig<Cfg = SerdeValue> {
    pub provider: String,
    pub name: String,
    pub config: Option<Cfg>,
    pub description: Option<String>,
}

pub type TcpServiceConfigWithLink = TcpServiceConfig<switchboard_custom_config::Link>;

impl TcpServiceConfigWithLink {
    pub async fn resolve_links<R>(
        self,
        resolver: &R,
    ) -> Result<TcpServiceConfig, switchboard_custom_config::Error>
    where
        R: switchboard_custom_config::LinkResolver,
    {
        if let Some(linked_config) = self.config {
            let resolved_config = resolver.fetch(&linked_config).await?.value;
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
