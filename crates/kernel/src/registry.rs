use std::{net::SocketAddr, sync::Arc};
use switchboard_model::TcpServiceConfig;
use switchboard_service::{
    registry::{ServiceProviderRegistry, ServiceProviderRegistryError},
    tcp::SharedTcpService,
};
mod handle;

pub use handle::*;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Service provider error: {0}")]
    ServiceProviderError(#[from] ServiceProviderRegistryError),
}

#[derive(Clone)]
pub struct Registry {
    pub registry: Arc<RwLock<ServiceProviderRegistry>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            registry: ServiceProviderRegistry::global(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("tcp fail to bind {bind}: {source}")]
pub struct TcpBindError {
    #[source]
    source: std::io::Error,
    bind: SocketAddr,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn create_tcp_service(
        &self,
        config: &TcpServiceConfig,
    ) -> Result<SharedTcpService, RegistryError> {
        let service = self
            .registry
            .read()
            .await
            .construct_tcp(&config.provider, config.config.clone())
            .await?;
        Ok(service)
    }
}
