use std::{collections::HashMap, path::Path};

use crate::kernel::{DiscoveredKernel, KernelDiscoveryError};

pub async fn scan_local_kernels(
    dir: &Path,
) -> Result<HashMap<String, DiscoveredKernel>, KernelDiscoveryError> {
    tracing::trace!("Scanning local kernels in directory {:?}", dir);
    if !dir.exists() {
        tracing::warn!(
            "Local kernel discovery directory {:?} does not exist, skipping local kernel discovery",
            dir
        );
        return Ok(HashMap::new());
    }
    let mut instances = HashMap::default();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let path = entry.path();
            if path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s == "run")
                && let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
            {
                tracing::trace!("Found local kernel {:?}", file_name);
                let file_content = std::fs::read(&path)?;
                let info: switchboard_model::discovery::DiscoveryInfo =
                    serde_json::from_slice(&file_content)?;
                let id = info.kernel.id.clone();
                let kernel = DiscoveredKernel {
                    addr: crate::kernel::KernelAddr::Grpc(info.connection.grpc.as_str().into()),
                    info,
                };
                instances.insert(id, kernel);
            }
        }
    }
    Ok(instances)
}
