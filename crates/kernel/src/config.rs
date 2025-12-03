use serde::{Deserialize, Serialize};

pub mod mem;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KernelConfig {
    pub controller: crate::controller::ControllerConfig, 
}