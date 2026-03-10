use std::{collections::HashSet, sync::Arc, time::Duration};

use registry::Registry;
use switchboard_file_resolver::FileResolver;
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

const DEFAULT_PREPARE_TTL_SECS: u64 = 60;

#[derive(Clone)]
pub(crate) struct PendingConfigTransaction {
    pub transaction_id: String,
    pub target_version: String,
    pub config: model::ServiceConfig,
    pub expires_at: std::time::Instant,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // #[error("Controller Connection error: {0}")]
    // ConfigServiceError(#[from] Box<crate::controller::ConnectError>),
    #[error("TCP Switchboard error: {0}")]
    TcpSwitchboardError(#[from] crate::switchboard::tcp::TcpSwitchboardError),
    #[error("Fetch service config error: {0}")]
    ResolveConfigFileError(#[from] crate::config::ResolveConfigFileError),
    #[error("config transaction not found: {0}")]
    ConfigTransactionNotFound(String),
    #[error("config transaction expired: {0}")]
    ConfigTransactionExpired(String),
    #[error("config transaction mismatch, expected {expected}, got {actual}")]
    ConfigTransactionMismatch { expected: String, actual: String },
    #[error("config version mismatch, expected {expected}, got {actual}")]
    ConfigVersionMismatch { expected: String, actual: String },

    #[error("Publish discovery error: {0}")]
    PublishDiscoveryError(#[from] crate::controller::discovery::PublishError),
    // #[error("Config service error: {0}")]
    // ConfigError(C::Error),
}

#[derive(Clone)]
pub struct KernelContext {
    pub(crate) registry: Registry,
    pub(crate) kernel_config: Arc<KernelConfig>,
    pub(crate) current_config: Arc<RwLock<model::ServiceConfig>>,
    pub(crate) tcp_switchboard: Arc<RwLock<TcpSwitchboard>>,
    // pub(crate) state: Arc<RwLock<KernelState>>,
    pub(crate) state: tokio::sync::watch::Sender<KernelState>,
    pub(crate) state_receiver: tokio::sync::watch::Receiver<KernelState>,
    // pub(crate) controller_handle: Arc<RwLock<Option<controller::listener::ListenerHandle>>>,
    pub(crate) controller_listener_handle:
        Arc<RwLock<Option<controller::listener::ListenerHandle>>>,
    pub(crate) pending_config_transaction: Arc<RwLock<Option<PendingConfigTransaction>>>,
    /// The handle for discovery publication, which can be used to unpublish on shutdown.
    pub(crate) discovery_handle: Arc<RwLock<Option<controller::discovery::PublishHandle>>>,
}

impl KernelContext {
    pub fn new(config: KernelConfig) -> Self {
        let (state, state_receiver) = tokio::sync::watch::channel(KernelState::init());
        Self {
            registry: Registry::new(),
            kernel_config: Arc::new(config),
            current_config: Arc::new(RwLock::new(model::ServiceConfig::default())),
            // controller_handle: Arc::new(tokio::sync::RwLock::new(None)),
            controller_listener_handle: Arc::new(tokio::sync::RwLock::new(None)),
            pending_config_transaction: Arc::new(tokio::sync::RwLock::new(None)),
            tcp_switchboard: Arc::new(RwLock::new(TcpSwitchboard::new_halted())),
            state,
            state_receiver,
            discovery_handle: Arc::new(RwLock::new(None)),
        }
    }
    pub fn get_state(&self) -> KernelState {
        use std::ops::Deref;
        self.state.borrow().deref().clone()
    }
    pub async fn fetch_config_locally(&self) -> Result<Option<model::ServiceConfig>, Error> {
        if let Some(config_path) = &self.kernel_config.config {
            if let Some(link) = config_path.as_link() {
                tracing::info!(
                    "Loading service config from file: {}",
                    link.to_string_lossy()
                );
            }
            let config = crate::config::fetch_config(config_path.clone(), &FileResolver).await?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
    pub async fn startup(&self) -> Result<(), Error> {
        let service_config = self.fetch_config_locally().await?;
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
            let listener_handle = self.spawn_controller_listener().await;
            *self.controller_listener_handle.write().await = Some(listener_handle);
        }
        // publish discovery
        {
            if let Some(me) = self.get_discovery_info() {
                self.publish_discovery(me).await?;
            } else {
                tracing::info!("No discovery info available, skipping discovery publication");
            }
        }
        Ok(())
    }
    /// Load and apply a service config to the running switchboard.
    ///
    /// # Errors
    /// Returns an error when switchboard router/listener update fails.
    pub async fn load_config(&self, sb_config: model::ServiceConfig) -> Result<(), Error> {
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
        if let Err(err) = self.state.send(state) {
            tracing::warn!("Failed to publish kernel state update: {}", err);
        }
    }

    pub async fn update_config(&self, sb_config: model::ServiceConfig) -> Result<(), Error> {
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

    /// Prepare a configuration transaction on this kernel.
    ///
    /// # Errors
    /// Returns an error when version does not match the config digest.
    pub async fn prepare_config(
        &self,
        transaction_id: String,
        config: model::ServiceConfig,
        expected_version: String,
        ttl_secs: Option<u64>,
    ) -> Result<(), Error> {
        {
            let pending = self.pending_config_transaction.read().await;
            if let Some(existing) = pending.as_ref() {
                if existing.transaction_id == transaction_id
                    && existing.target_version == expected_version
                {
                    return Ok(());
                }
                return Err(Error::ConfigTransactionMismatch {
                    expected: existing.transaction_id.clone(),
                    actual: transaction_id,
                });
            }
        }
        let target_version = config.digest_sha256_base64();
        if target_version != expected_version {
            return Err(Error::ConfigVersionMismatch {
                expected: expected_version,
                actual: target_version,
            });
        }
        let preparing_state = KernelState::new(KernelStateKind::Preparing {
            transaction_id: transaction_id.clone(),
            target_version: expected_version.clone(),
        });
        self.set_state(preparing_state);
        let ttl = ttl_secs.unwrap_or(DEFAULT_PREPARE_TTL_SECS);
        let pending = PendingConfigTransaction {
            transaction_id: transaction_id.clone(),
            target_version: expected_version.clone(),
            config,
            expires_at: std::time::Instant::now() + Duration::from_secs(ttl),
        };
        *self.pending_config_transaction.write().await = Some(pending);
        let prepared_state = KernelState::new(KernelStateKind::Prepared {
            transaction_id,
            target_version: expected_version,
        });
        self.set_state(prepared_state);
        Ok(())
    }

    /// Commit a previously prepared transaction and apply it.
    ///
    /// # Errors
    /// Returns an error when transaction is missing, expired, mismatched, or apply fails.
    pub async fn commit_config(
        &self,
        transaction_id: &str,
        expected_version: &str,
    ) -> Result<(), Error> {
        let config = {
            let mut pending_lock = self.pending_config_transaction.write().await;
            let pending = pending_lock
                .as_ref()
                .ok_or_else(|| Error::ConfigTransactionNotFound(transaction_id.to_string()))?;
            if pending.transaction_id != transaction_id {
                return Err(Error::ConfigTransactionMismatch {
                    expected: pending.transaction_id.clone(),
                    actual: transaction_id.to_string(),
                });
            }
            if pending.target_version != expected_version {
                return Err(Error::ConfigVersionMismatch {
                    expected: pending.target_version.clone(),
                    actual: expected_version.to_string(),
                });
            }
            if std::time::Instant::now() > pending.expires_at {
                return Err(Error::ConfigTransactionExpired(transaction_id.to_string()));
            }
            let config = pending.config.clone();
            pending_lock.take();
            config
        };
        self.set_state(KernelState::new(KernelStateKind::Committing {
            transaction_id: transaction_id.to_string(),
            target_version: expected_version.to_string(),
        }));
        self.update_config(config).await
    }

    /// Abort a prepared config transaction.
    ///
    /// # Errors
    /// Returns an error when transaction is missing or mismatched.
    pub async fn abort_config(&self, transaction_id: &str) -> Result<(), Error> {
        let mut pending_lock = self.pending_config_transaction.write().await;
        let pending = pending_lock
            .as_ref()
            .ok_or_else(|| Error::ConfigTransactionNotFound(transaction_id.to_string()))?;
        if pending.transaction_id != transaction_id {
            return Err(Error::ConfigTransactionMismatch {
                expected: pending.transaction_id.clone(),
                actual: transaction_id.to_string(),
            });
        }
        pending_lock.take();
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
