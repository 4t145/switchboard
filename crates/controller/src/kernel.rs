mod discovery;
use std::{collections::HashMap, net::SocketAddr};

pub use discovery::*;
mod connection;
pub use connection::*;
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum KernelAddr {
    Uds(std::path::PathBuf),
    Tcp(SocketAddr),
}

pub struct KernelManager {
    kernels: HashMap<KernelAddr, KernelHandle>,
}

impl KernelManager {
    pub fn new() -> Self {
        Self {
            kernels: HashMap::new(),
        }
    }
    pub fn add_new_kernel(&mut self, addr: KernelAddr) {
        tracing::debug!("Adding new kernel at addr: {:?}", addr);
        self.kernels.insert(addr.clone(), KernelHandle::new(addr));
    }
    pub async fn remove_kernel(&mut self, addr: &KernelAddr) {
        let handle = self.kernels.remove(addr);
        if let Some(handle) = handle {
            if let KernelHandleState::Connected(connected) = handle.state {
                let close_result = connected.transpose.close().await;
                if let Err(e) = close_result {
                    tracing::error!("Error closing connection to kernel at {:?}: {}", addr, e);
                }
            }
        }
    }
    pub async fn shutdown_all(&mut self) {
        let addrs: Vec<KernelAddr> = self.kernels.keys().cloned().collect();
        for addr in addrs {
            self.remove_kernel(&addr).await;
        }
    }
}

pub struct KernelHandle {
    pub addr: KernelAddr,
    pub state: KernelHandleState,
}

impl KernelHandle {
    pub fn new(addr: KernelAddr) -> Self {
        Self {
            addr,
            state: KernelHandleState::Disconnected,
        }
    }
    pub fn is_connected(&self) -> bool {
        matches!(self.state, KernelHandleState::Connected(_))
    }
    pub async fn take_over(
        &mut self,
        config: crate::config::KernelConfig,
        context: &crate::ControllerContext,
    ) -> Result<(), KernelConnectionError> {
        // check if already connected
        if self.is_connected() {
            tracing::info!("Kernel at {:?} is already connected", self.addr);
            return Ok(());
        }
        let mut transpose = self.addr.connect(config).await?;
        let info_and_state = transpose.take_over(context).await?;
        self.state = KernelHandleState::Connected(Connected {
            transpose,
            info_and_state,
        });
        Ok(())
    }
}

pub enum KernelHandleState {
    Disconnected,
    Connected(Connected),
}

pub struct Connected {
    pub transpose: Transpose,
    pub info_and_state: switchboard_model::kernel::KernelInfoAndState,
}
