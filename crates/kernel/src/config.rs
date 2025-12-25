use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
pub struct KernelConfig {
    pub info: switchboard_model::kernel::KernelInfo,
    pub controller: crate::controller::ControllerConfig,
    pub config: Option<LinkOrValue<FileStyleConfig>>,
}

use switchboard_custom_config::LinkOrValue;
pub use switchboard_model::resolve::fs::*;