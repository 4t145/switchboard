use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use switchboard_model::kernel::UDS_DEFAULT_PATH;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UdsListenerConfig {
    #[serde(default = "default_path")]
    pub path: PathBuf,
}

fn default_path() -> PathBuf {
    PathBuf::from(UDS_DEFAULT_PATH)
}

pub async fn listen_on_uds() {}
