use std::{collections::HashSet, sync::Arc};

use registry::Registry;
use switchboard_custom_config::fs::FsLinkResolver;
use switchboard_model::kernel::{KernelState, KernelStateKind};
use switchboard_service::tcp::TcpListener;

pub mod config;
pub mod controller;
pub mod registry;
pub mod switchboard;
pub mod tls;
pub use switchboard_model as model;
use tokio::sync::RwLock;

use crate::{config::KernelConfig, switchboard::tcp::TcpSwitchboard};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // #[error("Controller Connection error: {0}")]
    // ConfigServiceError(#[from] Box<crate::controller::ConnectError>),
    #[error("TCP Switchboard error: {0}")]
    TcpSwitchboardError(#[from] crate::switchboard::tcp::TcpSwitchboardError),
    #[error("Fetch service config error: {0}")]
    ResolveConfigFileError(#[from] crate::config::ResolveConfigFileError),
    // #[error("Config service error: {0}")]
    // ConfigError(C::Error),
}

#[derive(Clone)]
pub struct KernelContext {
    pub(crate) registry: Registry,
    pub(crate) kernel_config: Arc<KernelConfig>,
    pub(crate) current_config: Arc<RwLock<model::Config>>,
    pub(crate) tcp_switchboard: Arc<RwLock<TcpSwitchboard>>,
    // pub(crate) state: Arc<RwLock<KernelState>>,
    pub(crate) state: tokio::sync::watch::Sender<KernelState>,
    pub(crate) state_receiver: tokio::sync::watch::Receiver<KernelState>,
    // pub(crate) controller_handle: Arc<RwLock<Option<controller::listener::ListenerHandle>>>,
    pub(crate) controller_listener_handle:
        Arc<RwLock<Option<controller::listener::ListenerHandle>>>,
}

impl KernelContext {
    pub fn new(config: KernelConfig) -> Self {
        let (state, state_receiver) = tokio::sync::watch::channel(KernelState::init());
        Self {
            registry: Registry::new(),
            kernel_config: Arc::new(config),
            current_config: Arc::new(RwLock::new(model::Config::default())),
            // controller_handle: Arc::new(tokio::sync::RwLock::new(None)),
            controller_listener_handle: Arc::new(tokio::sync::RwLock::new(None)),
            tcp_switchboard: Arc::new(RwLock::new(TcpSwitchboard::new_halted())),
            state,
            state_receiver,
        }
    }
    pub fn get_state(&self) -> KernelState {
        use std::ops::Deref;
        self.state.borrow().deref().clone()
    }
    pub async fn startup(&self) -> Result<(), Error> {
        let service_config = {
            if let Some(config_path) = &self.kernel_config.config {
                if let Some(link) = config_path.as_link() {
                    tracing::info!("Loading service config from file: {}", link);
                }
                Some(crate::config::fetch_config(config_path.clone(), &FsLinkResolver).await?)
            } else {
                tracing::info!(
                    "No service config file specified, will waiting for controller to provide config"
                );
                None
            }
        };
        // start tcp switchboard
        {
            self.tcp_switchboard.write().await.ensure_running();
        }
        // load startup config
        {
            if let Some(sb_config) = service_config {
                self.load_config(sb_config).await?;
            }
        }
        // listen controller requests
        {
            self.spawn_controller_listener().await;
        }
        Ok(())
    }
    pub async fn load_config(&self, sb_config: model::Config) -> Result<(), Error> {
        // lock it up, make sure inner state unchanged during loading process
        let mut wg = self.tcp_switchboard.write().await;
        let tcp_switchboard = wg.handle_mut()?;
        let current_router = tcp_switchboard.get_current_router().await;
        let mut new_router = current_router.as_ref().clone();
        // for listeners
        {
            let existed = tcp_switchboard
                .tcp_listeners
                .keys()
                .cloned()
                .collect::<HashSet<_>>();
            let new_listeners = sb_config
                .tcp_listeners
                .keys()
                .cloned()
                .collect::<HashSet<_>>();
            let to_remove = existed.difference(&new_listeners);
            let to_add = new_listeners.difference(&existed);
            // remove old listeners
            for bind_addr in to_remove {
                tracing::info!(%bind_addr, "Removing TCP listener");
                tcp_switchboard.remove_listener_task(bind_addr).await;
                tracing::info!(%bind_addr, "Removed TCP listener");
            }
            // add new listeners
            for bind_addr in to_add {
                match TcpListener::bind(*bind_addr).await {
                    Ok(tcp_listener) => {
                        tracing::info!(%bind_addr, "Adding TCP listener");
                        tcp_switchboard.create_listener_task(tcp_listener).await?;
                        tracing::info!(%bind_addr, "Added TCP listener");
                    }
                    Err(e) => {
                        tracing::error!(%bind_addr, "Failed to bind TCP listener: {}", e);
                    }
                }
            }
        }
        // for routes
        {
            new_router.routes.clear();
            for (bind, route) in &sb_config.tcp_routes {
                new_router.routes.insert(*bind, route.into());
            }
        }
        // for tls
        {
            // lets just rebuild all tls
            new_router.tlss.clear();
            for (tls_name, tls) in &sb_config.tls {
                match crate::tls::build_tls_config(tls.clone()) {
                    Ok(tls) => {
                        new_router.tlss.insert(tls_name.as_str().into(), tls);
                    }
                    Err(e) => {
                        tracing::error!(%tls_name, "Failed to build TLS config: {}", e);
                    }
                }
            }
        }
        // for services
        {
            // lets just rebuild all services
            new_router.tcp_services.clear();
            for (service_name, service_config) in &sb_config.tcp_services {
                tracing::info!(%service_name, provider = %service_config.provider, "Creating TCP service");
                match self.registry.create_tcp_service(service_config).await {
                    Ok(service) => {
                        new_router
                            .tcp_services
                            .insert(service_name.as_str().into(), service);
                    }
                    Err(e) => {
                        tracing::error!(%service_name, "Failed to create TCP service: {}", e);
                    }
                }
            }
        }
        tcp_switchboard.update_router(new_router.into()).await?;
        // update current config
        {
            let mut current_config = self.current_config.write().await;
            *current_config = sb_config;
        }
        Ok(())
    }
    pub fn set_state(&self, state: KernelState) {
        {
            self.state
                .send(state)
                .expect("shouldn't fail since we hold a receiver");
        }
    }

    pub async fn update_config(&self, sb_config: model::Config) -> Result<(), Error> {
        let original_config_version = self.current_config.read().await.digest_sha256_base64();
        let new_config_version = sb_config.digest_sha256_base64();
        let new_state = KernelState::new(KernelStateKind::Updating {
            original_config_version,
            new_config_version: new_config_version.clone(),
        });
        self.set_state(new_state);
        self.load_config(sb_config).await?;
        let running_state = KernelState::new(KernelStateKind::Running {
            config_version: new_config_version,
        });
        self.set_state(running_state);
        Ok(())
    }
    pub async fn shutdown(&self) {
        self.set_state(KernelState::new(KernelStateKind::ShuttingDown));

        // shutdown supervisor
        tracing::info!("Shutting down TCP switchboard...");
        self.shutdown_tcp_switchboard().await;
        self.set_state(KernelState::new(KernelStateKind::Stopped));
        // shutdown controller listener
        tracing::info!("Shutting down controller listener...");
        self.shutdown_controller_listener().await;
        // shutdown controller
        // tracing::info!("Shutting down controller...");
        // self.shutdown_controller().await;
        tracing::info!("Kernel shutdown complete.");
    }
}
