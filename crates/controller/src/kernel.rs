mod discovery;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
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
    error::{ErrorStack, ResultObject},
    kernel::{KernelConnectionAndState, KernelInfoAndState},
};

const ROLLOUT_ABORT_REASON: &str = "all_or_nothing rollout failed";
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum KernelAddr {
    Uds(Arc<std::path::Path>),
    Grpc(Arc<str>),
}

impl Display for KernelAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelAddr::Uds(path) => write!(f, "unix://{}", path.display()),
            KernelAddr::Grpc(addr) => write!(f, "{}", addr),
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
                "http" | "https" | "grpc" => Ok(KernelAddr::Grpc(path.into())),
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
    pub async fn add_new_kernel(&mut self, kernel: DiscoveredKernel) {
        let addr = kernel.addr.clone();
        tracing::debug!(?kernel, "Adding new kernel at addr: {:?}", kernel.addr);
        let Ok(conn) = KernelGrpcConnection::connect(addr.clone())
            .await
            .inspect_err(|e| tracing::error!(?kernel, "cannot connect to addr {addr}: {e}"))
        else {
            self.kernels
                .insert(addr.clone(), KernelHandle::new_disconnected(addr));
            return;
        };
        self.kernels
            .insert(addr.clone(), KernelHandle::new_connected(addr, conn));
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
    pub async fn update_config(
        &self,
        new_config: switchboard_model::ServiceConfig,
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

    /// Prepare a configuration transaction on all tracked kernels.
    ///
    /// # Errors
    /// Each element may contain transport or kernel-side prepare errors.
    pub async fn prepare_config(
        &self,
        transaction_id: &str,
        new_config: switchboard_model::ServiceConfig,
    ) -> Vec<(KernelAddr, Result<(), KernelGrpcConnectionError>)> {
        let kernel_addrs = self.kernels.keys().cloned().collect::<Vec<_>>();
        self.prepare_config_for(transaction_id, new_config, &kernel_addrs)
            .await
    }

    /// Prepare a configuration transaction for selected kernels.
    ///
    /// # Errors
    /// Each element may contain transport or kernel-side prepare errors.
    pub async fn prepare_config_for(
        &self,
        transaction_id: &str,
        new_config: switchboard_model::ServiceConfig,
        kernel_addrs: &[KernelAddr],
    ) -> Vec<(KernelAddr, Result<(), KernelGrpcConnectionError>)> {
        let mut task_set = tokio::task::JoinSet::new();
        let new_config = Arc::new(new_config);
        let mut results = Vec::new();
        let selected = kernel_addrs.iter().cloned().collect::<HashSet<_>>();
        for addr in &selected {
            let Some(kernel) = self.kernels.get(addr) else {
                results.push((
                    addr.clone(),
                    Err(KernelGrpcConnectionError::KernelNotConnected),
                ));
                continue;
            };
            if let Some(handle) = kernel.get_connected_handle() {
                let addr = addr.clone();
                let config = new_config.clone();
                let transaction_id = transaction_id.to_string();
                let mut handle = handle.clone();
                task_set.spawn(async move {
                    handle
                        .prepare_config(&transaction_id, config.as_ref())
                        .map(|result| (addr, result))
                        .await
                });
            } else {
                results.push((
                    addr.clone(),
                    Err(KernelGrpcConnectionError::KernelNotConnected),
                ));
            }
        }
        results.extend(task_set.join_all().await);
        results
    }

    /// Commit a prepared configuration transaction on all tracked kernels.
    ///
    /// # Errors
    /// Each element may contain transport or kernel-side commit errors.
    pub async fn commit_config(
        &self,
        transaction_id: &str,
        version: &str,
    ) -> Vec<(KernelAddr, Result<(), KernelGrpcConnectionError>)> {
        let kernel_addrs = self.kernels.keys().cloned().collect::<Vec<_>>();
        self.commit_config_for(transaction_id, version, &kernel_addrs)
            .await
    }

    /// Commit a prepared configuration transaction for selected kernels.
    ///
    /// # Errors
    /// Each element may contain transport or kernel-side commit errors.
    pub async fn commit_config_for(
        &self,
        transaction_id: &str,
        version: &str,
        kernel_addrs: &[KernelAddr],
    ) -> Vec<(KernelAddr, Result<(), KernelGrpcConnectionError>)> {
        let mut task_set = tokio::task::JoinSet::new();
        let mut results = Vec::new();
        let selected = kernel_addrs.iter().cloned().collect::<HashSet<_>>();
        for addr in &selected {
            let Some(kernel) = self.kernels.get(addr) else {
                results.push((
                    addr.clone(),
                    Err(KernelGrpcConnectionError::KernelNotConnected),
                ));
                continue;
            };
            if let Some(handle) = kernel.get_connected_handle() {
                let addr = addr.clone();
                let transaction_id = transaction_id.to_string();
                let version = version.to_string();
                let mut handle = handle.clone();
                task_set.spawn(async move {
                    handle
                        .commit_config(&transaction_id, &version)
                        .map(|result| (addr, result))
                        .await
                });
            } else {
                results.push((
                    addr.clone(),
                    Err(KernelGrpcConnectionError::KernelNotConnected),
                ));
            }
        }
        results.extend(task_set.join_all().await);
        results
    }

    /// Abort a prepared configuration transaction for selected kernels.
    ///
    /// # Errors
    /// Each element may contain transport or kernel-side abort errors.
    pub async fn abort_config_for(
        &self,
        transaction_id: &str,
        kernel_addrs: &[KernelAddr],
    ) -> Vec<(KernelAddr, Result<(), KernelGrpcConnectionError>)> {
        let mut task_set = tokio::task::JoinSet::new();
        let mut results = Vec::new();
        for addr in kernel_addrs {
            let Some(kernel) = self.kernels.get(addr) else {
                results.push((
                    addr.clone(),
                    Err(KernelGrpcConnectionError::KernelNotConnected),
                ));
                continue;
            };
            if let Some(handle) = kernel.get_connected_handle() {
                let addr = addr.clone();
                let transaction_id = transaction_id.to_string();
                let mut handle = handle.clone();
                task_set.spawn(async move {
                    handle
                        .abort_config(&transaction_id, ROLLOUT_ABORT_REASON)
                        .map(|result| (addr, result))
                        .await
                });
            } else {
                results.push((
                    addr.clone(),
                    Err(KernelGrpcConnectionError::KernelNotConnected),
                ));
            }
        }
        results.extend(task_set.join_all().await);
        results
    }

    pub async fn shutdown_all(&mut self) {
        let addrs: Vec<KernelAddr> = self.kernels.keys().cloned().collect();
        for addr in addrs {
            self.remove_kernel(&addr).await;
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RolloutStatus {
    Succeeded,
    Failed { phase: &'static str },
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigRolloutReport {
    pub transaction_id: String,
    pub all_or_nothing: bool,
    pub status: RolloutStatus,
    pub prepare_results: Vec<(KernelAddr, ResultObject<()>)>,
    pub commit_results: Vec<(KernelAddr, ResultObject<()>)>,
    pub abort_results: Vec<(KernelAddr, ResultObject<()>)>,
    pub rollback_transaction_id: Option<String>,
    pub rollback_prepare_results: Vec<(KernelAddr, ResultObject<()>)>,
    pub rollback_commit_results: Vec<(KernelAddr, ResultObject<()>)>,
    pub rollback_abort_results: Vec<(KernelAddr, ResultObject<()>)>,
}
#[derive(Clone)]
pub struct KernelHandle {
    pub addr: KernelAddr,
    pub state: KernelHandleState,
}

impl KernelHandle {
    pub fn new_connected(addr: KernelAddr, conn: KernelGrpcConnection) -> Self {
        Self {
            addr,
            state: KernelHandleState::Connected(conn),
        }
    }
    pub fn new_disconnected(addr: KernelAddr) -> Self {
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
