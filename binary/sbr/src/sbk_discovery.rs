#[cfg(target_family = "unix")]
use std::collections::HashMap;

// 1. scan uds
// 2. scan k8s
#[cfg(target_family = "unix")]
static DEFAULT_SOCKET_DIR: &str = "/var/run/switchboard/";
#[derive(Debug, thiserror::Error)]
pub enum SbkDiscoveryError {
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
) -> Result<HashMap<String, SbkInstance>, SbkDiscoveryError> {
    let mut dir = tokio::fs::read_dir(socket_dir).await?;
    let mut instances = HashMap::default();
    while let Some(entry) = dir.next_entry().await? {
        use std::os::unix::fs::FileTypeExt;
        if entry.file_type().await?.is_socket() {
            let path = entry.path();
            let stem = path
                .file_stem()
                .ok_or_else(|| SbkDiscoveryError::SocketWithoutFileStem(path.clone()))?;
            instances.insert(stem.to_string_lossy().to_string(), SbkInstance::Uds(path));
        }
    }

    Ok(instances)
}

pub enum SbkInstance {
    Uds(std::path::PathBuf),
    Http(String),
}
