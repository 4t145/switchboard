use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use switchboard_model::kernel::KernelState;
use switchboard_service::{
    registry::{ServiceProviderRegistry, ServiceProviderRegistryError},
    tcp::{DynTcpService, RunningTcpService, TcpListener},
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
    #[error("Update service error: {0}")]
    UpdateServiceError(#[from] tokio::sync::watch::error::SendError<Arc<dyn DynTcpService>>),
}
#[derive(Clone)]
pub struct Supervisor {
    pub tcp_services: Arc<RwLock<HashMap<String, TcpServiceHandle>>>,
    pub registry: Arc<RwLock<ServiceProviderRegistry>>,
    pub state: Arc<RwLock<KernelState>>,
}

impl Default for Supervisor {
    fn default() -> Self {
        Self {
            tcp_services: Arc::new(RwLock::new(HashMap::new())),
            registry: ServiceProviderRegistry::global(),
            state: Arc::new(RwLock::new(KernelState::init())),
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

impl Supervisor {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn create_tcp_service(
        &self,
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
            .construct_tcp(&info.provider, info.config.clone())
            .await?;
        let tcp_listener = TcpListener::bind(info.bind, info.tls_config).await.map_err(|e|TcpBindError {
            source: e,
            bind: info.bind.clone()
        })?;
        let running_service = RunningTcpService::spawn(tcp_listener, service).await?;
        Ok(running_service)
    }
    pub async fn update_tcp_inner_service(
        &self,
        info: &TcpServiceInfo,
    ) -> Result<(), SupervisorError> {
        let tls_config = info
            .tls_config
            .clone()
            .map(crate::tls::build_tls_config)
            .transpose()?;
        let new_service = self
            .registry
            .read()
            .await
            .construct_tcp(&info.provider, info.config.clone(), tls_config)
            .await?;
        if let Some(old_service) = self.tcp_services.write().await.get(&info.id)
            && let Ok(old_service) = &old_service.service
        {
            old_service.update_service(new_service)?;
        }
        Ok(())
    }
    pub async fn recreate_tcp_service(&self, info: &TcpServiceInfo) -> Result<(), SupervisorError> {
        let tls_config = info
            .tls_config
            .clone()
            .map(crate::tls::build_tls_config)
            .transpose()?;
        let new_service = self
            .registry
            .read()
            .await
            .construct_tcp(&info.provider, info.config.clone(), tls_config)
            .await?;
        // shutdown old service so we can rebind
        if let Some(old_service) = self.tcp_services.write().await.remove(&info.id) {
            let _ = old_service
                .service?
                .cancel()
                .await
                .inspect_err(|e| tracing::error!("fail to join old TCP service: {e}"));
        }
        let running_service = new_service.bind(info.bind).await?;
        self.tcp_services.write().await.insert(
            info.id.clone(),
            TcpServiceHandle {
                service: Ok(running_service),
                info: info.clone(),
            },
        );
        Ok(())
    }
    pub async fn add_or_update_tcp_service(&self, info: TcpServiceInfo) {
        enum Action {
            Add,
            UpdateWholeService,
            UpdateInnerService,
            UpdateMetadata,
            Skip,
        }
        // check if service with same id exists
        let action = if let Some(service) = self.tcp_services.read().await.get(&info.id) {
            tracing::debug!(id = info.id, "TCP service with same ID already exists");
            if service.service.is_err() {
                tracing::info!(
                    id = info.id,
                    "Previous TCP service was in error state, recreating"
                );
                Action::UpdateWholeService
            } else {
                // check if the config is the same
                // should we rebind or just update inner service?
                // if bind or tls config has been changed, we need to rebind whole service
                let is_bind_changed = service.info.bind != info.bind;
                let is_tls_changed = service.info.tls_config != info.tls_config;
                let is_config_changed = service.info.config != info.config;
                let is_provider_changed = service.info.provider != info.provider;
                let is_name_changed = service.info.name != info.name;
                let is_bind_description_changed =
                    service.info.bind_description != info.bind_description;
                let is_service_description_changed =
                    service.info.service_description != info.service_description;
                if is_bind_changed || is_tls_changed {
                    tracing::debug!(
                        id = info.id,
                        is_bind_changed,
                        is_tls_changed,
                        "bind or tls config changed, need to rebind service"
                    );
                    Action::UpdateWholeService
                } else
                // check if other config has changed
                if is_config_changed || is_provider_changed {
                    tracing::info!(
                        id = info.id,
                        is_config_changed,
                        is_provider_changed,
                        "Updating inner TCP service configuration"
                    );
                    Action::UpdateInnerService
                } else if is_name_changed
                    || is_bind_description_changed
                    || is_service_description_changed
                {
                    tracing::info!(
                        id = info.id,
                        is_name_changed,
                        is_bind_description_changed,
                        is_service_description_changed,
                        "Updating TCP service metadata"
                    );
                    Action::UpdateMetadata
                } else {
                    tracing::info!(
                        id = info.id,
                        "No changes detected for TCP service, skipping update"
                    );
                    Action::Skip
                }
            }
        } else {
            tracing::info!(id = info.id, "Adding new TCP service");
            Action::Add
        };
        match action {
            Action::Skip => return,
            Action::UpdateInnerService => {
                if let Err(error) = self.update_tcp_inner_service(&info).await {
                    tracing::error!(id=info.id, %error, "Failed to update TCP service");
                } else {
                    // update metadata
                    if let Some(handle) = self.tcp_services.write().await.get_mut(&info.id) {
                        handle.info = info;
                    }
                }
            }
            Action::UpdateWholeService => {
                if let Err(error) = self.recreate_tcp_service(&info).await {
                    tracing::error!(id=info.id, %error, "Failed to recreate TCP service");
                }
            }
            Action::UpdateMetadata => {
                if let Some(handle) = self.tcp_services.write().await.get_mut(&info.id) {
                    handle.info = info;
                }
            }
            Action::Add => {
                let result = self
                    .create_tcp_service(&info)
                    .await
                    .inspect(|_| {
                        tracing::info!(id=info.id, name=info.name.as_deref().unwrap_or_default(), bind=%info.bind, "Created TCP service");
                    })
                    .inspect_err(|error| {
                        tracing::error!(id=info.id, name=info.name.as_deref().unwrap_or_default(), bind=%info.bind, %error, "Failed to create TCP service");
                    });
                self.tcp_services.write().await.insert(
                    info.id.clone(),
                    TcpServiceHandle {
                        service: result,
                        info,
                    },
                );
            }
        }
    }
    pub async fn remove_tcp_service(&self, id: &str) {
        if let Some(handle) = self.tcp_services.write().await.remove(id)
            && let Ok(service) = handle.service
        {
            tracing::info!(id = id, "Shutting down TCP service");
            if let Err(error) = service.cancel().await {
                tracing::error!(id = id, %error, "Failed to shutdown TCP service");
            } else {
                tracing::info!(id = id, "TCP service shut down successfully");
            }
        }
    }
    pub async fn shutdown(&self) {
        let mut task_set = tokio::task::JoinSet::new();
        for (_, handle) in self.tcp_services.write().await.drain() {
            if let Ok(service) = handle.service {
                task_set.spawn(service.cancel());
            }
        }
        task_set.join_all().await;
    }
}
