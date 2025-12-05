mod discovery;
use std::{collections::{BTreeMap, HashMap}, fmt::Display, net::SocketAddr};

pub use discovery::*;
mod connection;
pub use connection::*;
use switchboard_model::kernel::KernelConnectionAndState;
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum KernelAddr {
    Uds(std::path::PathBuf),
    Tcp(SocketAddr),
}

impl Display for KernelAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelAddr::Uds(path) => write!(f, "uds://{}", path.display()),
            KernelAddr::Tcp(addr) => write!(f, "tcp://{}", addr),
        }
    }
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
    pub async fn get_kernel_states(&self) -> BTreeMap<KernelAddr, KernelConnectionAndState> {
        let mut states = BTreeMap::new();
        for (addr, handle) in self.kernels.iter() {
            let state = match &handle.state {
                KernelHandleState::Disconnected => KernelConnectionAndState::Disconnected,
                KernelHandleState::Connected(handle) => {
                    KernelConnectionAndState::Connected(handle.get_info_and_state().await)
                }
            };
            states.insert(addr.clone(), state);
        }
        states
    }
    pub fn add_new_kernel(&mut self, addr: KernelAddr) {
        tracing::debug!("Adding new kernel at addr: {:?}", addr);
        self.kernels.insert(addr.clone(), KernelHandle::new(addr));
    }
    pub async fn remove_kernel(&mut self, addr: &KernelAddr) {
        let handle = self.kernels.remove(addr);
        if let Some(handle) = handle {
            if let KernelHandleState::Connected(connected) = handle.state {
                let close_result = connected.close().await;
                if let Err(e) = close_result {
                    tracing::error!("Error closing connection to kernel at {:?}: {}", addr, e);
                }
            }
        }
    }
    pub async fn disconnect_kernel(&mut self, addr: &KernelAddr) {
        if let Some(handle) = self.kernels.get_mut(addr) {
            let old_state = std::mem::replace(&mut handle.state, KernelHandleState::Disconnected);
            if let KernelHandleState::Connected(connected) = old_state {
                let close_result = connected.close().await;
                if let Err(e) = close_result {
                    tracing::error!("Error disconnecting kernel at {:?}: {}", addr, e);
                }
            }
        }
    }
    // this is specially used when kernel requests disconnection, we can close the connection from the task side, so need to avoid close it from here
    pub(crate) fn disconnect_kernel_without_close_connection(&mut self, addr: &KernelAddr) {
        if let Some(handle) = self.kernels.get_mut(addr) {
            let _ = std::mem::replace(&mut handle.state, KernelHandleState::Disconnected);
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
        let connection_handle =
            KernelConnectionHandle::spawn(transpose, self.addr.clone(), info_and_state, context);
        self.state = KernelHandleState::Connected(connection_handle);
        Ok(())
    }
    pub fn get_connected_handle(&self) -> Option<&KernelConnectionHandle> {
        match &self.state {
            KernelHandleState::Connected(handle) => Some(handle),
            _ => None,
        }
    }
}

pub enum KernelHandleState {
    Disconnected,
    Connected(KernelConnectionHandle),
}
