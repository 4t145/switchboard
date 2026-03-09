use std::collections::HashMap;

use crate::{ControllerContext, kernel::KernelAddr};
use switchboard_model::error::ResultObject;
use uuid::Uuid;

const PHASE_PREPARE: &str = "prepare";
const PHASE_COMMIT: &str = "commit";
const PHASE_ROLLBACK_PREPARE: &str = "rollback_prepare";
const PHASE_ROLLBACK_COMMIT: &str = "rollback_commit";

// 1. scan uds
// 2. scan k8s
#[cfg(target_family = "unix")]
#[derive(Debug, thiserror::Error)]
pub enum KernelDiscoveryError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("socket without file stem at path: {0}")]
    SocketWithoutFileStem(std::path::PathBuf),
}

#[cfg(target_family = "unix")]
pub async fn scan_uds(
    socket_dir: &std::path::Path,
) -> Result<HashMap<String, crate::kernel::KernelAddr>, KernelDiscoveryError> {
    // check if the socket dir exists
    if !socket_dir.exists() {
        tracing::warn!(
            "UDS socket dir {:?} does not exist, skipping UDS kernel discovery",
            socket_dir
        );
        return Ok(HashMap::new());
    }
    let mut dir = tokio::fs::read_dir(socket_dir).await?;
    let mut instances = HashMap::default();
    while let Some(entry) = dir.next_entry().await? {
        use std::os::unix::fs::FileTypeExt;
        if entry.file_type().await?.is_socket() {
            let path = entry.path();
            let stem = path
                .file_stem()
                .ok_or_else(|| KernelDiscoveryError::SocketWithoutFileStem(path.clone()))?;
            instances.insert(
                stem.to_string_lossy().to_string(),
                KernelAddr::Uds(path.as_path().into()),
            );
        }
    }

    Ok(instances)
}

impl ControllerContext {
    pub(crate) async fn discover_kernels(
        &self,
    ) -> Result<HashMap<String, crate::kernel::KernelAddr>, KernelDiscoveryError> {
        #[cfg(target_family = "unix")]
        {
            let uds_sockets = scan_uds(&self.controller_config.kernel.discovery.uds.dir).await?;
            Ok(uds_sockets)
        }
        #[cfg(not(target_family = "unix"))]
        {
            Ok(HashMap::new())
        }
    }
    pub async fn refresh_kernels(&self) -> Result<(), crate::Error> {
        let new_kernels = self.discover_kernels().await?;
        let new_kernel_keys = new_kernels
            .values()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let mut kernel_manager = self.kernel_manager.write().await;
        let existed_kernel_keys = kernel_manager
            .kernels
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let mut deleted_kernels = existed_kernel_keys
            .difference(&new_kernel_keys)
            .cloned()
            .collect::<Vec<_>>();
        for (_, addr) in new_kernels.iter() {
            if !kernel_manager.kernels.contains_key(addr) {
                kernel_manager.add_new_kernel(addr.clone());
            }
        }
        for addr in deleted_kernels.drain(..) {
            kernel_manager.remove_kernel(&addr).await;
        }
        Ok(())
    }
    pub async fn update_config(
        &self,
        new_config: switchboard_model::ServiceConfig,
    ) -> crate::kernel::ConfigRolloutReport {
        let previous_config = self.current_config.read().await.clone();
        let transaction_id = Uuid::now_v7().to_string();
        let version = new_config.digest_sha256_base64();
        let prepare_raw = self
            .kernel_manager
            .read()
            .await
            .prepare_config(&transaction_id, new_config.clone())
            .await;
        let prepared_kernel_addrs = prepare_raw
            .iter()
            .filter_map(|(addr, result)| result.as_ref().ok().map(|_| addr.clone()))
            .collect::<Vec<_>>();
        let prepare_results = prepare_raw
            .into_iter()
            .map(|(addr, result)| (addr, ResultObject::from(result)))
            .collect::<Vec<_>>();
        let prepare_failed = prepare_results
            .iter()
            .any(|(_, result)| matches!(result, ResultObject::Error(_)));
        if prepare_failed {
            let abort_results = self
                .kernel_manager
                .read()
                .await
                .abort_config_for(&transaction_id, &prepared_kernel_addrs)
                .await
                .into_iter()
                .map(|(addr, result)| (addr, ResultObject::from(result)))
                .collect::<Vec<_>>();
            return crate::kernel::ConfigRolloutReport {
                transaction_id,
                all_or_nothing: true,
                status: crate::kernel::RolloutStatus::Failed {
                    phase: PHASE_PREPARE,
                },
                prepare_results,
                commit_results: Vec::new(),
                abort_results,
                rollback_transaction_id: None,
                rollback_prepare_results: Vec::new(),
                rollback_commit_results: Vec::new(),
                rollback_abort_results: Vec::new(),
            };
        }
        let commit_raw = self
            .kernel_manager
            .read()
            .await
            .commit_config(&transaction_id, &version)
            .await;
        let commit_results = commit_raw
            .into_iter()
            .map(|(addr, result)| (addr, ResultObject::from(result)))
            .collect::<Vec<_>>();
        let committed_kernel_addrs = commit_results
            .iter()
            .filter_map(|(addr, result)| match result {
                ResultObject::Data(_) => Some(addr.clone()),
                ResultObject::Error(_) => None,
            })
            .collect::<Vec<_>>();
        let commit_failed = commit_results
            .iter()
            .any(|(_, result)| matches!(result, ResultObject::Error(_)));
        if commit_failed {
            let abort_results = self
                .kernel_manager
                .read()
                .await
                .abort_config_for(&transaction_id, &prepared_kernel_addrs)
                .await
                .into_iter()
                .map(|(addr, result)| (addr, ResultObject::from(result)))
                .collect::<Vec<_>>();
            let mut rollback_transaction_id = None;
            let mut rollback_prepare_results = Vec::new();
            let mut rollback_commit_results = Vec::new();
            let mut rollback_abort_results = Vec::new();
            let mut fail_phase = PHASE_COMMIT;
            if let Some(old_config) = previous_config {
                rollback_transaction_id = Some(format!("{}-rollback", Uuid::now_v7()));
                let rollback_txn = rollback_transaction_id.clone().unwrap_or_default();
                let rollback_version = old_config.digest_sha256_base64();
                let rollback_prepare_raw = self
                    .kernel_manager
                    .read()
                    .await
                    .prepare_config_for(&rollback_txn, old_config.clone(), &committed_kernel_addrs)
                    .await;
                let rollback_prepared_kernel_addrs = rollback_prepare_raw
                    .iter()
                    .filter_map(|(addr, result)| result.as_ref().ok().map(|_| addr.clone()))
                    .collect::<Vec<_>>();
                rollback_prepare_results = rollback_prepare_raw
                    .into_iter()
                    .map(|(addr, result)| (addr, ResultObject::from(result)))
                    .collect::<Vec<_>>();
                let rollback_prepare_failed = rollback_prepare_results
                    .iter()
                    .any(|(_, result)| matches!(result, ResultObject::Error(_)));
                if rollback_prepare_failed {
                    fail_phase = PHASE_ROLLBACK_PREPARE;
                    rollback_abort_results = self
                        .kernel_manager
                        .read()
                        .await
                        .abort_config_for(&rollback_txn, &rollback_prepared_kernel_addrs)
                        .await
                        .into_iter()
                        .map(|(addr, result)| (addr, ResultObject::from(result)))
                        .collect::<Vec<_>>();
                } else {
                    let rollback_commit_raw = self
                        .kernel_manager
                        .read()
                        .await
                        .commit_config_for(
                            &rollback_txn,
                            &rollback_version,
                            &rollback_prepared_kernel_addrs,
                        )
                        .await;
                    rollback_commit_results = rollback_commit_raw
                        .into_iter()
                        .map(|(addr, result)| (addr, ResultObject::from(result)))
                        .collect::<Vec<_>>();
                    let rollback_commit_failed = rollback_commit_results
                        .iter()
                        .any(|(_, result)| matches!(result, ResultObject::Error(_)));
                    if rollback_commit_failed {
                        fail_phase = PHASE_ROLLBACK_COMMIT;
                        rollback_abort_results = self
                            .kernel_manager
                            .read()
                            .await
                            .abort_config_for(&rollback_txn, &rollback_prepared_kernel_addrs)
                            .await
                            .into_iter()
                            .map(|(addr, result)| (addr, ResultObject::from(result)))
                            .collect::<Vec<_>>();
                    }
                }
            }
            return crate::kernel::ConfigRolloutReport {
                transaction_id,
                all_or_nothing: true,
                status: crate::kernel::RolloutStatus::Failed { phase: fail_phase },
                prepare_results,
                commit_results,
                abort_results,
                rollback_transaction_id,
                rollback_prepare_results,
                rollback_commit_results,
                rollback_abort_results,
            };
        }
        {
            let mut current_config_lock = self.current_config.write().await;
            *current_config_lock = Some(new_config);
        }
        crate::kernel::ConfigRolloutReport {
            transaction_id,
            all_or_nothing: true,
            status: crate::kernel::RolloutStatus::Succeeded,
            prepare_results,
            commit_results,
            abort_results: Vec::new(),
            rollback_transaction_id: None,
            rollback_prepare_results: Vec::new(),
            rollback_commit_results: Vec::new(),
            rollback_abort_results: Vec::new(),
        }
    }
}
