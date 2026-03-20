use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::ControllerContext;
use crate::utils::k8s::{K8sRuntimeEnvError, kube_client_if_in_cluster};

mod apply;
mod events;
mod reconcile;
mod watch;

pub use events::{ChangeKind, K8sRuntimeEvent, ObjectKey, ResourceKind};

const EVENT_CHANNEL_CAPACITY: usize = 2048;
const APPLY_CHANNEL_CAPACITY: usize = 64;

#[derive(Debug, Clone, Default)]
pub struct K8sApplyStatus {
    pub last_digest: Option<String>,
    pub last_apply_succeeded: bool,
    pub last_error: Option<String>,
}

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
    let watcher_context = watch::ResourceWatcherContext::new(ct.child_token(), tx.clone());
    let watcher_handles = watch::spawn_all_watchers(client, watcher_context);
    let (apply_tx, apply_rx) = mpsc::channel(APPLY_CHANNEL_CAPACITY);
    let apply_handle = apply::spawn_apply_worker(
        context.clone(),
        apply_rx,
        ct.child_token(),
        tx.clone(),
    );
    let _ = apply_tx.try_send(apply::ApplySignal::RebuildAll);

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
                let need_rebuild = apply::event_triggers_rebuild(&event);
                reconcile::dispatch_event(&context, event).await;
                if need_rebuild {
                    let _ = apply_tx.try_send(apply::ApplySignal::RebuildAll);
                }
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
    if let Err(err) = apply_handle.await {
        if !err.is_cancelled() {
            tracing::warn!(error = %err, "apply worker task join failed");
        }
    }

    loop_result
}
