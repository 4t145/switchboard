//! Interface modules for different controller communication methods
//!
//!
//!
//!

use serde::{Deserialize, Serialize};

use crate::ControllerContext;

pub mod http;
pub mod uds;
#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq, Default)]
pub struct InterfaceConfig {
    // Configuration options for different interfaces can be added here
    pub http: Option<http::HttpInterfaceConfig>,
}

impl ControllerContext {
    pub async fn start_up_all_interfaces(&self) -> crate::Result<()> {
        let interface_manager = &mut *self.interface_manager.write().await;
        if let Some(http_config) = &self.controller_config.interface.http
            && interface_manager.http_interface.is_none()
        {
            let http_interface = self.start_up_http_interface(http_config.clone()).await?;
            interface_manager.http_interface = Some(http_interface);
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct InterfaceManager {
    pub http_interface: Option<http::HttpInterface>,
}
