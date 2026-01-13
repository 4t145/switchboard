use serde::{Deserialize, Serialize};
use switchboard_custom_config::SerdeValue;
use switchboard_link_or_value::{LinkOrValue, Resolvable, Resolver};

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

impl<L, Cfg> Resolvable<L, Cfg, TcpServiceConfig<Cfg>> for TcpServiceConfig<LinkOrValue<L, Cfg>>
where
    L: Send + Sync + 'static,
    Cfg: Send + Sync + 'static,
{
    async fn resolve_with<R: Resolver<L, Cfg>>(
        self,
        resolver: &R,
    ) -> Result<TcpServiceConfig<Cfg>, R::Error> {
        let resolved_config = if let Some(linked_config) = self.config {
            let resolved = linked_config.resolve_with(resolver).await?;
            Some(resolved)
        } else {
            None
        };
        Ok(TcpServiceConfig {
            provider: self.provider,
            name: self.name,
            config: resolved_config,
            description: self.description,
        })
    }
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
