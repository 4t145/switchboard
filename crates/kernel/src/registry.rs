use std::{net::SocketAddr, sync::Arc};
use switchboard_model::TcpServiceConfig;
use switchboard_service::{
    TcpServiceProvider,
    registry::{ServiceProviderRegistry, ServiceProviderRegistryError},
    tcp::SharedTcpService,
};
mod handle;

pub use handle::*;
use tokio::sync::RwLock;

use crate::KernelContext;

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
    // pub async fn load_prelude(&self) {
    //     crate::register_prelude(&mut *self.registry.write().await);
    // }
}

impl KernelContext {
    pub async fn register_service<P: TcpServiceProvider>(&self, provider: P) {
        self.registry
            .registry
            .write()
            .await
            .register_tcp_provider(provider);
    }
}
