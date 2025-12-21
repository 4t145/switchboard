use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::KernelContext;

pub mod local;
pub mod tcp;
pub mod uds;

pub trait ConnectionStreamListener {
    type Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static;
    type Error: std::error::Error + Send + Sync + 'static;
    fn accept_next(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Self::Stream, Self::Error>> + Send + 'static;
    fn close(
        &mut self,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send + 'static;
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("tcp transport error: {0}")]
    TcpTransportError(#[from] tcp::TcpTransportError),
    #[error("uds transport error: {0}")]
    UdsTransportError(#[from] uds::UdsTransportError),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListenerConfig {
    pub uds: Option<uds::UdsListenerConfig>,
    pub tcp: Option<tcp::TcpListenerConfig>,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            uds: None,
            tcp: None,
        }
    }
}
pub struct ListenerHandle {
    pub ct: tokio_util::sync::CancellationToken,
    task_handle: tokio::task::JoinHandle<()>,
}

pub enum ListenerHandleQuitReason {
    Cancelled,
    Error(Box<dyn std::error::Error + Send + Sync>),
}
impl ListenerHandle {
    pub async fn run<L: ConnectionStreamListener>(
        mut listener: L,
        ct: CancellationToken,
    ) -> ListenerHandleQuitReason {
        let quit_reason = loop {
            let next_stream = tokio::select! {
                _ = ct.cancelled() => {
                    break ListenerHandleQuitReason::Cancelled;
                }
                accept_result = listener.accept_next() => {
                    match accept_result {
                        Ok(stream) =>stream,
                        Err(e) => {
                            break ListenerHandleQuitReason::Error(Box::new(e));
                        }
                    }
                }
            };
            use tonic::transport::Server;
            
        };

        if let ListenerHandleQuitReason::Error(e) = &quit_reason {
            tracing::warn!("listener encountered error and is shutting down: {}", e);
        }

        quit_reason
    }
}

impl KernelContext {
    pub async fn spawn_listener(&self) {
        let handle = ListenerHandle::spawn(self.clone());
        let old_handle = self
            .controller_listener_handle
            .write()
            .await
            .replace(handle);
        if let Some(old_handle) = old_handle {
            old_handle.shutdown().await;
        }
    }
    pub async fn shutdown_controller_listener(&self) {
        if let Some(handle) = self.controller_listener_handle.write().await.take() {
            handle.shutdown().await;
        }
    }
}
