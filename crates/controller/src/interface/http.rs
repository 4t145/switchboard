pub mod kernel_manager;

use std::net::SocketAddr;

use tokio_util::sync::CancellationToken;

use crate::ControllerContext;

pub struct HttpInterfaceConfig {
    pub bind: SocketAddr,
}

pub struct HttpInterface {
    pub config: HttpInterfaceConfig,
    pub ct: CancellationToken,
    pub handle: tokio::task::JoinHandle<()>,
}

#[derive(Clone)]
pub struct HttpState {
    pub controller_context: ControllerContext,
    
}