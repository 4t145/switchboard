mod discovery;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
};

pub use discovery::*;
mod connection;
pub mod grpc_client;
pub use connection::*;
use futures::FutureExt;
use serde::Serialize;
use switchboard_model::{
    error::ErrorStack,
    kernel::{KernelConnectionAndState, KernelInfoAndState},
};
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum KernelAddr {
    Uds(Arc<std::path::Path>),
    Http(Arc<str>),
}

impl Display for KernelAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelAddr::Uds(path) => write!(f, "unix://{}", path.display()),
            KernelAddr::Http(addr) => write!(f, "{}", addr),
        }
    }
}

impl Serialize for KernelAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for KernelAddr {
    type Err = KernelAddrParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((schema, path)) = s.split_once("://") {
            match schema {
                "uds" | "unix" => Ok(KernelAddr::Uds(
                    std::path::PathBuf::from(path).as_path().into(),
                )),
                "http" | "https" | "grpc" => Ok(KernelAddr::Http(path.into())),
                _ => Err(KernelAddrParseError::UnknownFormat {
                    format: schema.to_string(),
                }),
            }
        } else {
            Err(KernelAddrParseError::NoScheme)
        }
    }
}

impl<'de> serde::Deserialize<'de> for KernelAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        KernelAddr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KernelAddrParseError {
    #[error("unknown kernel address format: {format}")]
    UnknownFormat { format: String },
    #[error("missing scheme in kernel address")]
    NoScheme,
    #[error("invalid tcp address: {0}")]
    InvalidTcpAddress(#[from] std::net::AddrParseError),
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
        let mut task_set = tokio::task::JoinSet::new();
        for handle in self.kernels.values() {
            match &handle.state {
                KernelHandleState::Disconnected => {
                    states.insert(handle.addr.clone(), KernelConnectionAndState::Disconnected);
                }
                KernelHandleState::Connected(conn) => {
                    let addr = handle.addr.clone();
                    let mut conn = conn.clone();
                    let info = conn.get_info_from_cache();
                    let addr = addr.clone();
                    task_set.spawn(async move {
                        let result = conn.get_current_state().await;
                        (addr, info, result)
                    });
                }
            };
        }
        for (addr, info, result) in task_set.join_all().await {
            match result {
                Ok(state) => {
                    states.insert(
                        addr,
                        KernelConnectionAndState::Connected(KernelInfoAndState { info, state }),
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to get state from kernel at addr {:?}: {}", addr, e);
                    states.insert(
                        addr,
                        KernelConnectionAndState::FetchError(ErrorStack::from_std(e)),
                    );
                }
            }
        }
        states
    }
    pub fn add_new_kernel(&mut self, addr: KernelAddr) {
        tracing::debug!("Adding new kernel at addr: {:?}", addr);
        self.kernels.insert(addr.clone(), KernelHandle::new(addr));
    }
    pub async fn remove_kernel(&mut self, addr: &KernelAddr) {
        let handle = self.kernels.remove(addr);
        if let Some(handle) = handle
            && let KernelHandleState::Connected(connected) = handle.state
        {
            drop(connected);
        }
    }
    pub async fn disconnect_kernel(&mut self, addr: &KernelAddr) {
        if let Some(handle) = self.kernels.get_mut(addr) {
            let old_state = std::mem::replace(&mut handle.state, KernelHandleState::Disconnected);
            if let KernelHandleState::Connected(connected) = old_state {
                {
                    drop(connected);
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
    pub async fn update_config(
        &self,
        new_config: switchboard_model::Config,
    ) -> Vec<(KernelAddr, Result<(), KernelGrpcConnectionError>)> {
        let mut task_set = tokio::task::JoinSet::new();
        let new_config = std::sync::Arc::new(new_config);
        for (addr, kernel) in &self.kernels {
            if let Some(handle) = kernel.get_connected_handle() {
                let addr = addr.clone();
                let config = new_config.clone();
                let mut handle = handle.clone();
                task_set.spawn(async move {
                    handle
                        .update_config(config.as_ref())
                        .map(|result| (addr, result))
                        .await
                });
            }
        }
        task_set.join_all().await.into_iter().collect()
    }

    pub async fn shutdown_all(&mut self) {
        let addrs: Vec<KernelAddr> = self.kernels.keys().cloned().collect();
        for addr in addrs {
            self.remove_kernel(&addr).await;
        }
    }
}
#[derive(Clone)]
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

    pub fn get_connected_handle(&self) -> Option<KernelGrpcConnection> {
        match &self.state {
            KernelHandleState::Connected(conn) => Some(conn.clone()),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum KernelHandleState {
    Disconnected,
    Connected(KernelGrpcConnection),
}
