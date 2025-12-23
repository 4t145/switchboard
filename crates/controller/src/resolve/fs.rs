pub use switchboard_model::resolve::fs::*;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct FsResolveConfig {
    pub path: PathBuf,
}

impl Default for FsResolveConfig {
    fn default() -> Self {
        Self {
            path: default_switchboard_config_path(),
        }
    }
}
