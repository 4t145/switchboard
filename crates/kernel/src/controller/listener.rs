use serde::{Deserialize, Serialize};

pub mod uds;
pub mod ws;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListenerConfig {
    pub uds: Option<uds::UdsListenerConfig>,
}