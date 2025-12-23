use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
pub struct KernelConfig {
    pub info: switchboard_model::kernel::KernelInfo,
    pub controller: crate::controller::ControllerConfig,
    pub config: Option<PathBuf>,
}

pub use switchboard_model::resolve::fs::*;