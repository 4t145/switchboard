use std::sync::Arc;
use tokio::sync::RwLock;
use crate::kernel::KernelDiscoveryError;
pub mod config;
pub mod kernel;

#[derive(Clone)]
pub struct ControllerContext {
    pub controller_config: Arc<config::ControllerConfig>,
    pub kernel_manager: Arc<RwLock<kernel::KernelManager>>,
}

impl ControllerContext {
    pub fn new(controller_config: config::ControllerConfig) -> Self {
        Self {
            controller_config: controller_config.into(),
            kernel_manager: Arc::new(RwLock::new(kernel::KernelManager::new())),
        }
    }
    pub async fn startup(&self) -> Result<(), Error> {
        self.refresh_kernels().await?;
        Ok(())
    }
    pub async fn shutdown(&self) -> Result<(), Error> {
        // shutdown all kernel connections
        {
            let mut manager = self.kernel_manager.write().await;
            manager.shutdown_all().await;
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Kernel discovery error: {0}")]
    KernelDiscoveryError(#[from] KernelDiscoveryError),
    #[error("Kernel connection error: {0}")]
    KernelConnectionError(#[from] crate::kernel::KernelConnectionError),
}
