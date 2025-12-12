use crate::kernel::KernelDiscoveryError;
use std::sync::Arc;
use tokio::sync::RwLock;
pub mod config;
pub mod interface;
pub mod kernel;
pub mod resource;
pub mod dir;
pub const DEFAULT_NAMESPACE: &str = "switchboard";
#[derive(Clone)]
pub struct ControllerContext {
    pub controller_config: Arc<config::ControllerConfig>,
    pub kernel_manager: Arc<RwLock<kernel::KernelManager>>,
    pub interface_manager: Arc<RwLock<interface::InterfaceManager>>,
    pub k8s_client: Option<kube::Client>,
}

impl ControllerContext {
    pub fn new(controller_config: config::ControllerConfig) -> Self {
        Self {
            controller_config: controller_config.into(),
            kernel_manager: Arc::new(RwLock::new(kernel::KernelManager::new())),
            interface_manager: Arc::new(RwLock::new(interface::InterfaceManager::default())),
            k8s_client: None,
        }
    }
    pub async fn try_init_k8s_client(&mut self) -> Result<(), > {
        let client = kube::Client::try_default().await?;
        self.k8s_client = Some(client);
        Ok(())
    }
    pub fn get_k8s_client(&self) -> Option<kube::Client> {
        self.k8s_client.as_ref().cloned()
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
    KernelConnectionError(#[from] crate::kernel::KernelConnectionError),
    #[error("Startup http interface error: {0}")]
    StartupHttpInterfaceError(#[source] std::io::Error),

    #[error("Kubernetes client error: {0}")]
    KubernetesClientError(#[from] kube::Error),
}
