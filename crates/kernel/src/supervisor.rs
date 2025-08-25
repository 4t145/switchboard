use std::{collections::HashMap, sync::Arc};
use switchboard_service::{
    registry::{ServiceProviderRegistry, ServiceProviderRegistryError},
    tcp::RunningTcpService,
};
mod handle;

pub use handle::*;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("Service provider error: {0}")]
    ServiceProviderError(#[from] ServiceProviderRegistryError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TLS build error: {0}")]
    TlsBuildError(#[from] crate::tls::TlsBuildError),
}

pub struct Supervisor {
    pub tcp_services: HashMap<String, TcpServiceHandle>,
    pub registry: Arc<RwLock<ServiceProviderRegistry>>,
}

impl Default for Supervisor {
    fn default() -> Self {
        Self {
            tcp_services: HashMap::new(),
            registry: ServiceProviderRegistry::global(),
        }
    }
}

impl Supervisor {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn create_tcp_service(
        &mut self,
        info: &TcpServiceInfo,
    ) -> Result<RunningTcpService, SupervisorError> {
        let tls_config = info
            .tls_config
            .clone()
            .map(crate::tls::build_tls_config)
            .transpose()?;
        let service = self
            .registry
            .read()
            .await
            .construct_tcp(&info.provider, info.config.clone(), tls_config)
            .await?;
        let running_service = service.bind(info.bind).await?;
        Ok(running_service)
    }
    pub async fn add_tcp_service(&mut self, info: TcpServiceInfo) {
        let result = self
            .create_tcp_service(&info)
            .await
            .inspect(|_| {
                tracing::info!(id=info.id, name=info.name.as_deref().unwrap_or_default(), bind=%info.bind, "Created TCP service");
            })
            .inspect_err(|error| {
                tracing::error!(id=info.id, name=info.name.as_deref().unwrap_or_default(), bind=%info.bind, %error, "Failed to create TCP service");
            });
        self.tcp_services.insert(
            info.id.clone(),
            TcpServiceHandle {
                service: result,
                info,
            },
        );
    }
    pub async fn shutdown(&mut self) {
        let mut task_set = tokio::task::JoinSet::new();
        for (_, handle) in self.tcp_services.drain() {
            if let Ok(service) = handle.service {
                task_set.spawn(service.cancel());
            }
        }
        task_set.join_all().await;
    }
}
