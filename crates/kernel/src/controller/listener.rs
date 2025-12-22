use crate::KernelContext;
use serde::{Deserialize, Serialize};
use tokio::net;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

// pub mod local;
pub mod tcp;
pub mod uds;

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
    join_set: tokio::task::JoinSet<(std::result::Result<(), tonic::transport::Error>)>,
}

pub enum ListenerHandleQuitReason {
    Cancelled,
    Error(Box<dyn std::error::Error + Send + Sync>),
}
impl ListenerHandle {
    pub async fn shutdown(mut self) {
        self.ct.cancel();
        while let Some(res) = self.join_set.join_next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    tracing::error!("Controller listener taskjoined with error: {}", e);
                }
                Err(e) => {
                    tracing::error!("Controller listener task join error: {}", e);
                }
            }
        }
    }
}

impl KernelContext {
    pub async fn spawn_listener(&self) -> ListenerHandle {
        let ct = tokio_util::sync::CancellationToken::new();
        let grpc_server = self.build_grpc_server();
        let listener_config = &self.kernel_config.controller.listen;
        let mut join_set = tokio::task::JoinSet::new();
        'bind_tcp: {
            if let Some(tcp_config) = &listener_config.tcp {
                let addr: std::net::SocketAddr = (tcp_config.host, tcp_config.port).into();
                let bind_result = tokio::net::TcpListener::bind(addr).await;
                let listener = match bind_result {
                    Err(e) => {
                        tracing::error!(
                            "Failed to bind controller tcp listener on {}: {}",
                            addr,
                            e
                        );
                        break 'bind_tcp;
                    }
                    Ok(listener) => listener,
                };
                let incoming = tokio_stream::wrappers::TcpListenerStream::new(
                    tokio::net::TcpListener::bind(addr).await.unwrap(),
                );
                tracing::info!("Controller gRPC listening on tcp://{}", addr);
                let span = tracing::info_span!("controller-tcp-listener", %addr);
                join_set.spawn(
                    tonic::transport::Server::builder()
                        .add_service(grpc_server.clone())
                        .serve_with_incoming_shutdown(incoming, ct.child_token().cancelled_owned())
                        .instrument(span),
                );
            }
        }
        'bind_uds: {
            if let Some(uds_config) = &listener_config.uds {
                let path = &uds_config.path;
                let path_display = path.to_string_lossy();
                let listener = match tokio::net::UnixListener::bind(path.clone()) {
                    Err(e) => {
                        tracing::error!(
                            "Failed to bind controller uds listener on {}: {}",
                            path_display,
                            e
                        );
                        break 'bind_uds;
                    }
                    Ok(listener) => listener,
                };
                let incoming = tokio_stream::wrappers::UnixListenerStream::new(listener);
                tracing::info!("Controller gRPC listening on uds://{}", path_display);
                let span = tracing::info_span!("controller-uds-listener", path = %path_display);
                join_set.spawn(
                    tonic::transport::Server::builder()
                        .add_service(grpc_server)
                        .serve_with_incoming_shutdown(incoming, ct.child_token().cancelled_owned())
                        .instrument(span),
                );
            }
        }
        ListenerHandle { ct, join_set }
    }
    pub async fn shutdown_controller_listener(&self) {
        if let Some(handle) = self.controller_listener_handle.write().await.take() {
            handle.shutdown().await;
        }
    }
}
