use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::ControllerContext;
use crate::utils::k8s::{K8sRuntimeEnvError, kube_client_if_in_cluster};

mod events;
mod reconcile;
mod watch;

pub use events::{ChangeKind, K8sRuntimeEvent, ObjectKey, ResourceKind};

const EVENT_CHANNEL_CAPACITY: usize = 2048;

#[derive(Debug, thiserror::Error)]
pub enum K8sRuntimeError {
    #[error("k8s runtime environment error: {0}")]
    Env(#[from] K8sRuntimeEnvError),
    #[error("event loop channel closed unexpectedly")]
    EventChannelClosed,
    #[error("k8s runtime task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
}

pub struct K8sRuntimeHandle {
    ct: CancellationToken,
    task: JoinHandle<Result<(), K8sRuntimeError>>,
}

impl std::fmt::Debug for K8sRuntimeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("K8sRuntimeHandle").finish_non_exhaustive()
    }
}

impl K8sRuntimeHandle {
    pub async fn cancel(self) -> Result<(), K8sRuntimeError> {
        self.ct.cancel();
        self.task.await??;
        Ok(())
    }

    pub fn abort(&self) {
        self.ct.cancel();
        self.task.abort();
    }
}

pub async fn watch_event_loop(
    context: ControllerContext,
    ct: CancellationToken,
) -> Result<Option<K8sRuntimeHandle>, K8sRuntimeError> {
    let Some(client) = kube_client_if_in_cluster().await? else {
        tracing::warn!(
            "trying to run in k8s mode out of the k8s environment, going to exit event loop"
        );
        return Ok(None);
    };
    let task_ct = ct.clone();
    let task = tokio::spawn(async move { run_event_loop(context, client, task_ct).await });

    Ok(Some(K8sRuntimeHandle { ct, task }))
}

async fn run_event_loop(
    context: ControllerContext,
    client: kube::Client,
    ct: CancellationToken,
) -> Result<(), K8sRuntimeError> {
    let (tx, mut rx) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
    let watcher_context = watch::ResourceWatcherContext::new(ct.child_token(), tx);
    let watcher_handles = watch::spawn_all_watchers(client, watcher_context);

    let mut loop_result = Ok(());
    loop {
        tokio::select! {
            _ = ct.cancelled() => {
                break;
            }
            next_event = rx.recv() => {
                let Some(event) = next_event else {
                    loop_result = Err(K8sRuntimeError::EventChannelClosed);
                    break;
                };
                reconcile::dispatch_event(&context, event).await;
            }
        }
    }

    ct.cancel();
    for watcher in watcher_handles {
        if let Err(err) = watcher.await {
            if err.is_cancelled() {
                continue;
            }
            tracing::warn!(error = %err, "watcher task join failed");
        }
    }

    loop_result
}
