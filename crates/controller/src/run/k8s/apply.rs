use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

use crate::ControllerContext;
use crate::run::k8s::{K8sApplyStatus, K8sRuntimeEvent};
use switchboard_model::switchboard_serde_value::value;

const APPLY_DEBOUNCE_MILLIS: u64 = 300;
const K8S_RESOLVER_NAME: &str = "k8s";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplySignal {
    RebuildAll,
}

pub fn event_triggers_rebuild(event: &K8sRuntimeEvent) -> bool {
    matches!(event, K8sRuntimeEvent::ResourceChanged { .. })
}

pub fn spawn_apply_worker(
    context: ControllerContext,
    mut rx: mpsc::Receiver<ApplySignal>,
    ct: CancellationToken,
    event_tx: mpsc::Sender<K8sRuntimeEvent>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        tracing::info!("k8s apply worker started");

        loop {
            let first_signal = tokio::select! {
                _ = ct.cancelled() => break,
                next = rx.recv() => next,
            };

            let Some(_signal) = first_signal else {
                break;
            };

            let debounce = sleep(Duration::from_millis(APPLY_DEBOUNCE_MILLIS));
            tokio::pin!(debounce);
            loop {
                tokio::select! {
                    _ = ct.cancelled() => return,
                    _ = &mut debounce => break,
                    next = rx.recv() => {
                        if next.is_none() {
                            return;
                        }
                    }
                }
            }

            apply_rebuild_all(&context, &event_tx).await;
        }

        tracing::info!("k8s apply worker stopped");
    })
}

async fn apply_rebuild_all(context: &ControllerContext, event_tx: &mpsc::Sender<K8sRuntimeEvent>) {
    let k8s_resolve_config = context.controller_config.resolve.k8s.clone();
    let resolve_value = value!({
        "watch_all_namespaces": k8s_resolve_config.watch_all_namespaces,
        "gateway_namespaces": k8s_resolve_config.gateway_namespaces,
        "gateway_namespace": k8s_resolve_config.gateway_namespace,
    });

    let resolved = match context
        .resolve_config(K8S_RESOLVER_NAME, resolve_value)
        .await
    {
        Ok(config) => config,
        Err(err) => {
            tracing::warn!(error = %err, "k8s apply resolve failed");
            set_apply_status(
                context,
                K8sApplyStatus {
                    last_digest: None,
                    last_apply_succeeded: false,
                    last_error: Some(format!("resolve failed: {err}")),
                },
            )
            .await;
            let _ = event_tx.send(K8sRuntimeEvent::ApplyStatusChanged).await;
            return;
        }
    };

    let link_resolver = context.clone().link_resolver();
    let standard_config = match resolved.resolve_into_standard(&link_resolver).await {
        Ok(config) => config,
        Err(err) => {
            tracing::warn!(error = %err, "k8s apply config conversion failed");
            set_apply_status(
                context,
                K8sApplyStatus {
                    last_digest: None,
                    last_apply_succeeded: false,
                    last_error: Some(format!("resolve_into_standard failed: {err}")),
                },
            )
            .await;
            let _ = event_tx.send(K8sRuntimeEvent::ApplyStatusChanged).await;
            return;
        }
    };

    let digest = standard_config.digest_sha256_base64();
    {
        let status = context.k8s_apply_status.read().await;
        if status.as_ref().is_some_and(|s| {
            s.last_apply_succeeded && s.last_digest.as_deref() == Some(digest.as_str())
        }) {
            tracing::debug!(digest = %digest, "k8s apply skipped because digest unchanged");
            return;
        }
    }

    let report = context.update_config(standard_config).await;
    let apply_succeeded = matches!(report.status, crate::kernel::RolloutStatus::Succeeded);
    if apply_succeeded {
        tracing::info!(digest = %digest, transaction_id = %report.transaction_id, "k8s apply succeeded");
    } else {
        tracing::warn!(digest = %digest, transaction_id = %report.transaction_id, status = ?report.status, "k8s apply failed");
    }

    set_apply_status(
        context,
        K8sApplyStatus {
            last_digest: Some(digest),
            last_apply_succeeded: apply_succeeded,
            last_error: if apply_succeeded {
                None
            } else {
                Some(format!("rollout failed: {:?}", report.status))
            },
        },
    )
    .await;
    let _ = event_tx.send(K8sRuntimeEvent::ApplyStatusChanged).await;
}

async fn set_apply_status(context: &ControllerContext, status: K8sApplyStatus) {
    *context.k8s_apply_status.write().await = Some(status);
}
