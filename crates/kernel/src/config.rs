use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
pub struct KernelConfig {
    pub info: switchboard_model::kernel::KernelInfo,
    pub controller: crate::controller::ControllerConfig,
    pub config: Option<LinkOrValue<FileStyleConfig>>,
}

pub use switchboard_model::resolve::file_style::*;
