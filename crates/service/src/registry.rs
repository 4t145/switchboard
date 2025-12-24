use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use crate::SerdeValue;
use tokio::sync::RwLock;

use crate::{
    BoxTcpServiceProvider,
    BoxedError,
    DynTcpServiceProvider,
    TcpServiceProvider,
    tcp::SharedTcpService, // tcp::{DynTcpService, tls::TlsService},
};

pub struct ServiceProviderRegistry {
    pub tcp: HashMap<String, BoxTcpServiceProvider>,
}
#[derive(Debug, thiserror::Error)]
pub enum ServiceProviderRegistryError {
    #[error("Service provider not found: {0}")]
    ServiceProviderNotFound(String),
    #[error("Fail to construct service: {0}")]
    ConstructError(#[from] BoxedError),
}

impl ServiceProviderRegistry {
    pub async fn construct_tcp(
        &self,
        name: &str,
        config: Option<SerdeValue>,
    ) -> Result<SharedTcpService, ServiceProviderRegistryError> {
        let provider = self.tcp.get(name).ok_or_else(|| {
            ServiceProviderRegistryError::ServiceProviderNotFound(name.to_owned())
        })?;
        let service = provider.construct(config).await?;
        Ok(service)
    }
    pub fn register_tcp_provider<P: TcpServiceProvider>(&mut self, p: P) {
        self.tcp.insert(
            P::NAME.to_string(),
            Box::new(p) as Box<dyn DynTcpServiceProvider>,
        );
    }
    pub fn unregister_tcp_provider(&mut self, name: &str) {
        self.tcp.remove(name);
    }
    pub fn get_tcp_provider(
        &self,
        name: &str,
    ) -> Result<&dyn DynTcpServiceProvider, ServiceProviderRegistryError> {
        self.tcp
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| ServiceProviderRegistryError::ServiceProviderNotFound(name.to_string()))
    }
    pub fn global() -> Arc<RwLock<Self>> {
        static INSTANCE: OnceLock<Arc<RwLock<ServiceProviderRegistry>>> = OnceLock::new();
        INSTANCE
            .get_or_init(|| {
                Arc::new(RwLock::new(ServiceProviderRegistry {
                    tcp: HashMap::new(),
                }))
            })
            .clone()
    }
}
