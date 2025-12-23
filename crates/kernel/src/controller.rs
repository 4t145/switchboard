pub mod grpc_service;
pub mod listener;
use serde::{Deserialize, Serialize};
use switchboard_model::protocol::DEFAULT_STATE_REPORT_INTERVAL_SECS;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ControllerConfig {
    pub state_report_interval: u32,
    pub listen: listener::ListenerConfig,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            state_report_interval: DEFAULT_STATE_REPORT_INTERVAL_SECS,
            listen: listener::ListenerConfig::default(),
        }
    }
}
