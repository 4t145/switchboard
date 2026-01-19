use crate::kernel::KernelDiscoveryError;
use std::sync::Arc;
use tokio::sync::RwLock;
pub mod config;
pub mod dir;
pub mod interface;
pub mod kernel;
pub mod link_resolver;
pub mod resolve;
pub mod resource;
pub mod storage;
pub const DEFAULT_NAMESPACE: &str = "switchboard";
#[derive(Clone)]
pub struct ControllerContext {
    pub controller_config: Arc<config::ControllerConfig>,
    pub kernel_manager: Arc<RwLock<kernel::KernelManager>>,
    pub interface_manager: Arc<RwLock<interface::InterfaceManager>>,
    pub storage: storage::SharedStorage,
    pub resolve: Arc<resolve::ServiceConfigResolverRegistry>,
    pub current_config: Arc<RwLock<Option<switchboard_model::ServiceConfig>>>,
}

impl ControllerContext {
    pub async fn new(controller_config: config::ControllerConfig) -> Result<Self> {
        let this = Self {
            storage: storage::create_storage(&controller_config.storage).await?,
            kernel_manager: Arc::new(RwLock::new(kernel::KernelManager::new())),
            interface_manager: Arc::new(RwLock::new(interface::InterfaceManager::default())),
            controller_config: controller_config.into(),
            resolve: resolve::ServiceConfigResolverRegistry::prelude().into(),
            current_config: Arc::new(RwLock::new(None)),
        };
        Ok(this)
    }
    pub async fn startup(&self) -> Result<()> {
        self.start_up_all_interfaces().await?;
        self.refresh_kernels().await?;
        Ok(())
    }
    pub async fn shutdown(&self) -> Result<()> {
        // shutdown all kernel connections
        {
            let mut manager = self.kernel_manager.write().await;
            manager.shutdown_all().await;
        }
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Kernel discovery error: {0}")]
    KernelDiscoveryError(#[from] KernelDiscoveryError),
    #[error("Kernel connection error: {0}")]
    KernelConnectionError(#[from] crate::kernel::KernelGrpcConnectionError),
    #[error("Startup http interface error: {0}")]
    StartupHttpInterfaceError(#[source] std::io::Error),

    #[error("Kubernetes client error: {0}")]
    KubernetesClientError(#[from] kube::Error),

    #[error("Storage error: {0}")]
    StorageError(#[from] crate::storage::StorageError),

    #[error("Resolve config error: {0}")]
    ResolveServiceConfigError(#[from] crate::resolve::ResolveServiceConfigError),

    #[error("Json Interpreter error: {0}")]
    JsonInterpreterError(#[from] crate::storage::JsonInterpreterError),

    #[error("Link resolve error: {0}")]
    LinkResolveError(#[from] crate::link_resolver::LinkResolveError),
}
