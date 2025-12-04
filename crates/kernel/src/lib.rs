use std::sync::Arc;

use supervisor::{Supervisor, TcpServiceInfo};
use switchboard_http::HttpProvider;
use switchboard_model::{
    NamedService, ServiceDescriptor,
    kernel::{KernelState, KernelStateKind},
};
use switchboard_pf::PortForwardProvider;
use switchboard_service::registry::ServiceProviderRegistry;
use switchboard_socks5::Socks5Provider;
use switchboard_uds::UdsProvider;

pub mod config;
pub mod controller;
pub mod supervisor;
pub mod tls;
pub use switchboard_model as model;

use crate::config::KernelConfig;

pub fn register_prelude(registry: &mut ServiceProviderRegistry) {
    // Register the prelude services
    registry.register_tcp_provider(Socks5Provider);
    registry.register_tcp_provider(PortForwardProvider);
    registry.register_tcp_provider(HttpProvider);
    registry.register_tcp_provider(UdsProvider);
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Controller Connection error: {0}")]
    ConfigServiceError(#[from] crate::controller::ConnectError),
    // #[error("Config service error: {0}")]
    // ConfigError(C::Error),
}

#[derive(Clone)]
pub struct KernelContext {
    pub supervisor: Supervisor,
    pub kernel_config: Arc<KernelConfig>,
    pub controller_handle: Arc<tokio::sync::RwLock<Option<controller::ControllerHandle>>>,
    pub listener_handle: Arc<tokio::sync::RwLock<Option<controller::listener::ListenerHandle>>>,
}

impl KernelContext {
    pub fn new(config: KernelConfig) -> Self {
        Self {
            supervisor: Supervisor::new(),
            kernel_config: Arc::new(config),
            controller_handle: Arc::new(tokio::sync::RwLock::new(None)),
            listener_handle: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }
    pub async fn get_state(&self) -> KernelState {
        use std::ops::Deref;
        self.supervisor.state.read().await.deref().clone()
    }
    pub async fn startup(&self) -> Result<(), Error> {
        self.spawn_listener().await;
        self.load_config(self.kernel_config.startup.clone()).await?;
        Ok(())
    }
    pub async fn load_config(&self, sb_config: model::Config) -> Result<(), Error> {
        let _registry = self.supervisor.registry.read().await;
        for (id, bind) in sb_config.get_enabled() {
            tracing::info!(%id, %bind, "Adding bind to supervisor");
            let sd = &bind.service;
            let service_info = match sd {
                ServiceDescriptor::Anon(anon_service_descriptor) => {
                    let mut tls_config = None;
                    if let Some(tls_name) = &anon_service_descriptor.tls {
                        tls_config = sb_config.get_tls(tls_name);
                    }

                    TcpServiceInfo {
                        id: id.to_owned(),
                        bind: bind.addr,
                        bind_description: bind.description.clone(),
                        config: anon_service_descriptor.config.clone(),
                        provider: anon_service_descriptor.provider.clone(),
                        name: None,
                        service_description: None,
                        tls_config: tls_config.cloned(),
                    }
                }
                ServiceDescriptor::Named(name) => {
                    let Some(NamedService {
                        provider,
                        name,
                        config,
                        description,
                        tls,
                    }) = sb_config.get_named_service(name)
                    else {
                        tracing::error!(%id, %name, "Failed to get named service");
                        continue;
                    };
                    let mut tls_config = None;
                    if let Some(tls_name) = tls {
                        tls_config = sb_config.get_tls(tls_name);
                    }
                    TcpServiceInfo {
                        id: id.to_owned(),
                        bind: bind.addr,
                        bind_description: bind.description.clone(),
                        config: config.clone(),
                        provider: provider.clone(),
                        name: Some(name.clone()),
                        service_description: description.clone(),
                        tls_config: tls_config.cloned(),
                    }
                }
            };
            self.supervisor
                .add_or_update_tcp_service(service_info)
                .await;
        }
        // remove disabled binds

        let current_ids: Vec<_> = self
            .supervisor
            .tcp_services
            .read()
            .await
            .keys()
            .cloned()
            .collect();
        for id in current_ids {
            if !sb_config.enabled.contains(&id) {
                tracing::info!(%id, "Removing disabled bind from supervisor");
                self.supervisor.remove_tcp_service(&id).await;
            }
        }
        Ok(())
    }
    pub async fn set_state(&self, state: KernelState) -> Result<(), Error> {
        use std::ops::DerefMut;
        {
            let mut current_state = self.supervisor.state.write().await;
            *current_state.deref_mut() = state.clone();
        }
        if let Some(controller_handle) = &*self.controller_handle.read().await {
            controller_handle.update_state(state).await?;
        }
        Ok(())
    }
    pub fn sign_config(&self, config: &model::Config) -> Vec<u8> {
        config.sign(&self.kernel_config.controller.connect.psk)
    }

    pub async fn update_config(&self, sb_config: model::Config) -> Result<(), Error> {
        let original_config_signature = self.sign_config(&self.kernel_config.startup);
        let new_config_signature = self.sign_config(&sb_config);
        let new_state = KernelState::new(KernelStateKind::Updating {
            original_config_signature: original_config_signature,
            new_config_signature: new_config_signature.clone(),
        });
        self.set_state(new_state).await?;
        self.load_config(sb_config).await?;
        let running_state = KernelState::new(KernelStateKind::Running {
            config_signature: new_config_signature,
        });
        self.set_state(running_state).await?;
        Ok(())
    }
    pub async fn shutdown(&self) {
        self.set_state(KernelState::new(KernelStateKind::ShuttingDown)).await.ok();
        // shutdown supervisor
        self.supervisor.shutdown().await;
        self.set_state(KernelState::new(KernelStateKind::Stopped)).await.ok();
        // shutdown controller listener
        self.shutdown_listener().await;
        // shutdown controller
        self.shutdown_controller().await;
    }
}
