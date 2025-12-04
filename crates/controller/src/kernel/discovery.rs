use std::collections::HashMap;


use crate::{ControllerContext, kernel::KernelAddr};

// 1. scan uds
// 2. scan k8s
#[cfg(target_family = "unix")]
#[derive(Debug, thiserror::Error)]
pub enum KernelDiscoveryError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("socket not found at path: {0}")]
    SocketNotFound(std::path::PathBuf),
    #[error("socket without file stem at path: {0}")]
    SocketWithoutFileStem(std::path::PathBuf),
}

#[cfg(target_family = "unix")]
pub async fn scan_uds(
    socket_dir: &std::path::Path,
) -> Result<HashMap<String, crate::kernel::KernelAddr>, KernelDiscoveryError> {
    let mut dir = tokio::fs::read_dir(socket_dir).await?;
    let mut instances = HashMap::default();
    while let Some(entry) = dir.next_entry().await? {
        use std::os::unix::fs::FileTypeExt;
        if entry.file_type().await?.is_socket() {
            let path = entry.path();
            let stem = path
                .file_stem()
                .ok_or_else(|| KernelDiscoveryError::SocketWithoutFileStem(path.clone()))?;
            instances.insert(stem.to_string_lossy().to_string(), KernelAddr::Uds(path));
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
    pub async fn take_over_kernel(
        &self,
        addr: crate::kernel::KernelAddr,
    ) -> Result<(), crate::Error> {
        let mut kernel_manager = self.kernel_manager.write().await;
        if let Some(kernel_handle) = kernel_manager.kernels.get_mut(&addr) {
            kernel_handle
                .take_over(self.controller_config.kernel.clone(), self)
                .await?;
        }
        Ok(())
    }
    pub async fn take_over_all_kernels(&self) -> Result<(), crate::Error> {
        let mut kernel_manager = self.kernel_manager.write().await;
        for (_, kernel_handle) in kernel_manager.kernels.iter_mut() {
            if kernel_handle.is_connected() {
                continue;
            }
            kernel_handle
                .take_over(self.controller_config.kernel.clone(), self)
                .await?;
        }
        Ok(())
    }
}
