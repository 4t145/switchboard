use std::path::PathBuf;

use serde::Deserialize;
use switchboard_link_or_value::{LinkOrValue};
#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
pub struct KernelConfig {
    pub info: switchboard_model::kernel::KernelInfo,
    pub controller: crate::controller::ControllerConfig,
    pub provider: ProviderConfig,
    pub config: Option<LinkOrValue<PathBuf, SerdeValue>>,
}

use switchboard_model::SerdeValue;
pub use switchboard_model::resolve::file_style::*;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ProviderConfig {
    pub http: HttpProviderConfig
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct HttpProviderConfig {
    pub plugins: Vec<String>
}