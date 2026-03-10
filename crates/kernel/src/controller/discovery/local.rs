//! Discovery in uds
//!
//! As well as kernel is running, it will create a .run file in a specific path, and controller can discover it by watching the path.
//!
//! When kernel programs shut down, it will remove the .run file, so controller can know the kernel is gone.
//!
//! In the file we just write `DiscoveryInstance` in json format

use std::path::{Path, PathBuf};

use switchboard_model::discovery::DiscoveryInfo;
use tokio::{fs::File, io::AsyncWriteExt};
#[derive(Debug, thiserror::Error)]
pub enum LocalPublishError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct LocalPublisher {
    pub path: PathBuf,
}

impl LocalPublisher {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

pub struct LocalPublishHandle {
    pub file: File,
    pub path: PathBuf,
}

pub async fn publish(
    dir: &Path,
    me: DiscoveryInfo,
) -> Result<LocalPublishHandle, LocalPublishError> {
    let run_file_content = serde_json::to_vec(&me)?;
    tokio::fs::create_dir_all(dir).await?;
    let file_name = format!("{}.run", me.kernel.id);
    let tmp_name = format!(".{}.tmp-{}", file_name, std::process::id());
    let tmp_path = dir.join(tmp_name);
    let path = dir.join(&file_name);
    let mut f = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)
        .await?;
    f.write_all(&run_file_content).await?;
    f.sync_all().await?;
    tokio::fs::rename(&tmp_path, &path).await?;
    Ok(LocalPublishHandle {
        file: f,
        path: path.to_owned(),
    })
}

pub async fn unpublish(handle: LocalPublishHandle) -> Result<(), LocalPublishError> {
    handle.file.sync_all().await?;
    {
        // Drop handle
        handle.file;
    };
    tokio::fs::remove_file(&handle.path).await?;
    Ok(())
}

impl super::Publish for LocalPublisher {
    type Error = LocalPublishError;
    type Handle = LocalPublishHandle;
    fn publish(
        &self,
        me: DiscoveryInfo,
    ) -> impl std::future::Future<Output = Result<Self::Handle, Self::Error>> + Send {
        let path = self.path.clone();
        async move { publish(&path, me).await }
    }
    fn unpublish(
        &self,
        handle: Self::Handle,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        async move { unpublish(handle).await }
    }
}
