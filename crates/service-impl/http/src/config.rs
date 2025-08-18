use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{HttpVersion, flow::build::FlowConfig};

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub flow_config: FlowConfig,
    pub server: ServerConfig,
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub version: HttpVersion,
}
