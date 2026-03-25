pub mod k8s;
pub mod standalone;

use crate::ControllerContext;
use tokio_util::sync::CancellationToken;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RunMode {
    /// Under standalone mode, controller receive requests and take actions
    Standalone,
    /// Under K8s mode, contoller watch k8s resource and take actions
    K8s,
}

impl RunMode {
    pub fn is_k8s(&self) -> bool {
        matches!(self, RunMode::K8s)
    }
}

impl ControllerContext {
    pub async fn spawn_k8s_runtime(&self) -> crate::Result<()> {
        let handle = k8s::watch_event_loop(self.clone(), CancellationToken::new()).await?;
        *self.k8s_runtime.write().await = handle;
        Ok(())
    }

    pub async fn cancel_k8s_runtime(&self) -> crate::Result<()> {
        if let Some(runtime_handle) = self.k8s_runtime.write().await.take() {
            runtime_handle.cancel().await?;
        }
        Ok(())
    }
}
