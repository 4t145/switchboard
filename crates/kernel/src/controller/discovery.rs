use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use switchboard_model::discovery::DiscoveryInfo;

use crate::KernelContext;

pub mod local;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DiscoveryConfig {
    local: Option<PathBuf>,
}

impl Publisher {
    pub fn from_config(config: DiscoveryConfig) -> Self {
        Publisher {
            local: config.local.map(local::LocalPublisher::new),
        }
    }
}

pub struct Publisher {
    pub local: Option<local::LocalPublisher>,
}

pub struct PublishHandle {
    pub local: Option<local::LocalPublishHandle>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            #[cfg(target_family = "unix")]
            local: Some(switchboard_model::kernel::RUN_FILE_DEFAULT_DIR.into()),
        }
    }
}

// Publish object should be a pure data struct
pub trait Publish {
    type Error: std::error::Error + Send + Sync + 'static;
    type Handle: Send + Sync + 'static;
    fn publish(
        &self,
        me: DiscoveryInfo,
    ) -> impl Future<Output = Result<Self::Handle, Self::Error>> + Send;
    fn unpublish(
        &self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Fail to publish locally: {0}")]
    Local(#[from] local::LocalPublishError),

    #[error("Unsupported discovery publisher")]
    Unsupported,
}

impl Publish for Publisher {
    type Error = PublishError;
    type Handle = PublishHandle;

    fn publish(
        &self,
        me: DiscoveryInfo,
    ) -> impl Future<Output = Result<Self::Handle, Self::Error>> + Send {
        let local = self.local.clone();
        async move {
            let local_handle = if let Some(local) = local {
                Some(local.publish(me.clone()).await?)
            } else {
                None
            };
            Ok(PublishHandle {
                local: local_handle,
            })
        }
    }

    fn unpublish(
        &self,
        handle: Self::Handle,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        let local = self.local.clone();
        async move {
            if let Some(local) = local {
                if let Some(local_handle) = handle.local {
                    local.unpublish(local_handle).await?;
                }
            }
            Ok(())
        }
    }
}
impl KernelContext {
    pub async fn publish_discovery(&self, me: DiscoveryInfo) -> Result<(), crate::Error> {
        tracing::info!("Publishing discovery info: {:?}", me);
        let mut handle_wg = self.discovery_handle.write().await;
        if let Some(old_handle) = handle_wg.take() {
            // If there is an existing handle, unpublish it first to avoid leaving stale discovery info.
            let publisher = Publisher::from_config(self.kernel_config.controller.discovery.clone());
            publisher.unpublish(old_handle).await?;
        }
        let publisher = Publisher::from_config(self.kernel_config.controller.discovery.clone());
        let handle = publisher.publish(me).await?;
        handle_wg.replace(handle);
        Ok(())
    }
    pub async fn unpublish_discovery(&self) -> Result<(), crate::Error> {
        let mut handle_wg = self.discovery_handle.write().await;
        if let Some(handle) = handle_wg.take() {
            let publisher = Publisher::from_config(self.kernel_config.controller.discovery.clone());
            publisher.unpublish(handle).await?;
        }
        Ok(())
    }
    pub fn get_discovery_info(&self) -> Option<DiscoveryInfo> {
        let endpoint = self
            .kernel_config
            .controller
            .listen
            .http
            .as_ref()?
            .get_endpoint();
        Some(DiscoveryInfo {
            connection: switchboard_model::discovery::DiscoveryConnectionInfo { grpc: endpoint },
            kernel: self.kernel_config.info.clone(),
        })
    }
}
