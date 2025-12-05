use crate::kernel::KernelDiscoveryError;
use std::sync::Arc;
use tokio::sync::RwLock;
pub mod config;
pub mod interface;
pub mod kernel;

#[derive(Clone)]
pub struct ControllerContext {
    pub controller_config: Arc<config::ControllerConfig>,
    pub kernel_manager: Arc<RwLock<kernel::KernelManager>>,
    pub interface_manager: Arc<RwLock<interface::InterfaceManager>>,
}

impl ControllerContext {
    pub fn new(controller_config: config::ControllerConfig) -> Self {
        Self {
            controller_config: controller_config.into(),
            kernel_manager: Arc::new(RwLock::new(kernel::KernelManager::new())),
            interface_manager: Arc::new(RwLock::new(interface::InterfaceManager::default())),
        }
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
}
