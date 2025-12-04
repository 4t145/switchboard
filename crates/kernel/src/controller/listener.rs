use serde::{Deserialize, Serialize};

use crate::KernelContext;

pub mod local;
pub mod tcp;
pub mod uds;
pub mod ws;

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

impl ListenerHandle {
    pub async fn shutdown(self) {
        self.ct.cancel();
        let _ = self.task_handle.await;
    }
    pub fn spawn(context: KernelContext) -> Self {
        let ct = tokio_util::sync::CancellationToken::new();
        let ct_child = ct.child_token();
        let listener_config = context
            .kernel_config
            .controller
            .listen
            .clone()
            .unwrap_or_default();
        let task_handle = tokio::spawn(async move {
            let uds_listener = if let Some(uds_config) = &listener_config.uds {
                tracing::info!(
                    "starting UDS controller listener at {}",
                    uds_config.path.display()
                );
                uds::UdsListener::new(uds_config.clone())
                    .await
                    .inspect_err(|e| {
                        tracing::error!(
                            "failed to start UDS controller listener at {}: {}",
                            uds_config.path.display(),
                            e
                        );
                    })
                    .ok()
            } else {
                None
            };
            let tcp_listener = if let Some(tcp_config) = &listener_config.tcp {
                tracing::info!(
                    "starting TCP controller listener at {}:{}",
                    tcp_config.host,
                    tcp_config.port
                );
                tcp::TcpListener::new(tcp_config.clone())
                    .await
                    .inspect_err(|e| {
                        tracing::error!(
                            "failed to start TCP controller listener at {}:{}: {}",
                            tcp_config.host,
                            tcp_config.port,
                            e
                        );
                    })
                    .ok()
            } else {
                None
            };
            loop {
                tokio::select! {
                    _ = ct_child.cancelled() => {
                        tracing::info!("controller listener task is cancelled, shutting down");
                        break;
                    }
                    accept_uds_result = async {
                        if let Some(uds_listener) = &uds_listener {
                            uds_listener.accept().await
                        } else {
                            futures::future::pending().await
                        }
                    } => {
                        match accept_uds_result {
                            Ok(controller_connection) => {
                                tracing::info!("accepted new controller connection");
                                if let Err(e) = context.spawn_controller_connection_event_loop(controller_connection).await {
                                    tracing::error!("failed to spawn controller connection event loop: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("failed to accept controller connection: {}", e);
                            }
                        }
                    }
                    accept_tcp_result = async {
                        if let Some(tcp_listener) = &tcp_listener {
                            tcp_listener.accept().await
                        } else {
                            futures::future::pending().await
                        }
                    } => {
                        match accept_tcp_result {
                            Ok(controller_connection) => {
                                tracing::info!("accepted new controller connection");
                                if let Err(e) = context.spawn_controller_connection_event_loop(controller_connection).await {
                                    tracing::error!("failed to spawn controller connection event loop: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("failed to accept controller connection: {}", e);
                            }
                        }
                    }
                }
            }
        });
        ListenerHandle { ct, task_handle }
    }
}

impl KernelContext {
    pub async fn spawn_listener(&self) {
        let handle = ListenerHandle::spawn(self.clone());
        let old_handle = self.listener_handle.write().await.replace(handle);
        if let Some(old_handle) = old_handle {
            old_handle.shutdown().await;
        }
    }
    pub async fn shutdown_listener(&self) {
        if let Some(handle) = self.listener_handle.write().await.take() {
            handle.shutdown().await;
        }
    }
}
