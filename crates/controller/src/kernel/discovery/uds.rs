#![cfg(target_family = "unix")]

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::kernel::KernelDiscoveryError;

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct KernelDiscoveryUdsConfig {
    pub dir: PathBuf,
    pub scan_interval_secs: u32,
    pub max_frame_size: u32,
}

impl Default for KernelDiscoveryUdsConfig {
    fn default() -> Self {
        KernelDiscoveryUdsConfig {
            dir: PathBuf::from(switchboard_model::kernel::RUN_FILE_DEFAULT_DIR),
            scan_interval_secs: 10,
            max_frame_size: 1 << 22,
        }
    }
}

#[deprecated = "uds socket not supported yet"]
pub async fn scan_uds(
    _socket_dir: &std::path::Path,
) -> Result<HashMap<String, super::DiscoveredKernel>, KernelDiscoveryError> {
    // check if the socket dir exists
    // if !socket_dir.exists() {
    //     tracing::warn!(
    //         "UDS socket dir {:?} does not exist, skipping UDS kernel discovery",
    //         socket_dir
    //     );
    //     return Ok(HashMap::new());
    // }
    // let mut dir = tokio::fs::read_dir(socket_dir).await?;
    // let mut instances = HashMap::default();
    // while let Some(entry) = dir.next_entry().await? {
    //     use std::os::unix::fs::FileTypeExt;
    //     if entry.file_type().await?.is_socket() {
    //         let path = entry.path();
    //         let stem = path
    //             .file_stem()
    //             .ok_or_else(|| KernelDiscoveryError::SocketWithoutFileStem(path.clone()))?;
    //         instances.insert(
    //             stem.to_string_lossy().to_string(),
    //             super::DiscoveredKernel {
    //                 addr: crate::kernel::KernelAddr::Uds(path.as_path().into()),
    //                 info: unimplemented!("TODO: uds not supported yet"),
    //             },
    //         );
    //     }
    // }

    return Ok(HashMap::new());
}
