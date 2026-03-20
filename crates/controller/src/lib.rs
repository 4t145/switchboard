use std::sync::Arc;
use tokio::sync::RwLock;
pub mod config;
pub mod dir;
pub mod interface;
pub mod kernel;
pub mod link_resolver;
pub mod resolve;
pub mod resource;
pub mod run;
pub mod storage;
pub mod utils;
pub const DEFAULT_NAMESPACE: &str = "switchboard";

#[derive(Clone)]
pub struct ControllerContext {
    pub controller_config: Arc<config::ControllerConfig>,
    pub kernel_manager: Arc<RwLock<kernel::KernelManager>>,
    pub interface_manager: Arc<RwLock<interface::InterfaceManager>>,
    pub storage: storage::SharedStorage,
    pub resolve: Arc<resolve::ServiceConfigResolverRegistry>,
    pub current_config: Arc<RwLock<Option<switchboard_model::ServiceConfig>>>,
    pub scan_task: Arc<RwLock<Option<kernel::ScanTaskHandle>>>,
    pub k8s_runtime: Arc<RwLock<Option<run::k8s::K8sRuntimeHandle>>>,
    pub k8s_apply_status: Arc<RwLock<Option<run::k8s::K8sApplyStatus>>>,
    pub run_mode: Arc<RwLock<Option<run::RunMode>>>,
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
            scan_task: Arc::new(RwLock::new(None)),
            k8s_runtime: Arc::new(RwLock::new(None)),
            k8s_apply_status: Arc::new(RwLock::new(None)),
            run_mode: Arc::new(RwLock::new(None)),
        };
        Ok(this)
    }
    pub async fn startup(&self, run_mode: run::RunMode) -> Result<()> {
        self.start_up_all_interfaces().await?;
        self.refresh_kernels().await?;
        self.spawn_scan_task().await;
        if run_mode.is_k8s() {
            self.spawn_k8s_runtime().await?;
        }
        *self.run_mode.write().await = Some(run_mode);
        Ok(())
    }
    pub async fn shutdown(&self) -> Result<()> {
        self.cancel_k8s_runtime().await?;
        self.cancel_scan_task().await;
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
    KernelDiscoveryError(#[from] crate::kernel::KernelDiscoveryError),
    #[error("Kernel connection error: {0}")]
    KernelConnectionError(#[from] crate::kernel::KernelGrpcConnectionError),
    #[error("Startup http interface error: {0}")]
    StartupHttpInterfaceError(#[source] std::io::Error),

    #[error("Kubernetes client error: {0}")]
    KubernetesClientError(#[from] kube::Error),

    #[error("Kubernetes runtime environment error: {0}")]
    KubernetesRuntimeEnvError(#[from] crate::utils::k8s::K8sRuntimeEnvError),

    #[error("Kubernetes runtime loop error: {0}")]
    KubernetesRuntimeLoopError(#[from] crate::run::k8s::K8sRuntimeError),

    #[error("Storage error: {0}")]
    StorageError(#[from] crate::storage::StorageError),

    #[error("Resolve config error: {0}")]
    ResolveServiceConfigError(#[from] crate::resolve::ResolveServiceConfigError),

    #[error("Resolve config file error: {0}")]
    ResolveConfigFileError(#[from] switchboard_model::resolve::file_style::ResolveConfigFileError),

    #[error("Json Interpreter error: {0}")]
    JsonInterpreterError(#[from] crate::storage::JsonInterpreterError),

    #[error("Link resolve error: {0}")]
    LinkResolveError(#[from] crate::link_resolver::LinkResolveError),

    #[error("File browser io error: {0}")]
    FileBrowserIoError(#[source] std::io::Error),

    #[error("File browser path not allowed: {0}")]
    FileBrowserPathNotAllowed(std::path::PathBuf),

    #[error("Controller is not running in kubernetes cluster")]
    NotInKubernetesCluster,
    
    #[error("Controller is running in kubernetes cluster")]
    InKubernetesCluster,
}
