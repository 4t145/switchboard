use std::path::PathBuf;
use tokio_stream::wrappers::UnixListenerStream;

use serde::{Deserialize, Serialize};
use switchboard_model::{
    control::{ControllerMessage, KernelMessage},
    kernel::UDS_DEFAULT_PATH,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UdsListenerConfig {
    #[serde(default = "default_path")]
    pub path: PathBuf,
}

fn default_path() -> PathBuf {
    PathBuf::from(UDS_DEFAULT_PATH)
}

pub async fn listen_on_uds() {}