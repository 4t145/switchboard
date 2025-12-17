use serde::{Deserialize, Serialize};

pub mod mem;
pub mod file;
#[derive(Clone, Debug, Deserialize)]
pub struct KernelConfig {
    #[serde(default)]
    pub info: switchboard_model::kernel::KernelInfo,
    #[serde(default)]
    pub controller: crate::controller::ControllerConfig, 
    // #[serde(with = "file")] 
    // pub startup: switchboard_model::Config,
}
