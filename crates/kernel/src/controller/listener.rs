use crate::KernelContext;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

use switchboard_custom_config::fs::FsLinkResolver;
use tracing::Instrument;

// pub mod local;
pub mod http;
pub mod uds;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ListenerConfig {
    pub uds: Option<uds::UdsListenerConfig>,
    pub http: Option<http::HttpListenerConfig>,
}

pub struct ListenerHandle {
    pub ct: tokio_util::sync::CancellationToken,
    join_set: tokio::task::JoinSet<std::result::Result<(), tonic::transport::Error>>,
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
    pub async fn spawn_controller_listener(&self) -> ListenerHandle {
        let ct = tokio_util::sync::CancellationToken::new();
        let grpc_server = self.build_grpc_server();
        let listener_config = &self.kernel_config.controller.listen;
        let mut join_set = tokio::task::JoinSet::new();
        'bind_http: {
            if let Some(http_config) = &listener_config.http {
                let addr: std::net::SocketAddr = (http_config.host, http_config.port).into();
                let mut tls_acceptor = None;
                if let Some(tls) = &http_config.tls {
                    match tls.resolver.clone().resolve(&FsLinkResolver).await {
                        Ok(tls_resolver) => {
                            let tls_option = tls.options.clone();
                            let tls = switchboard_model::Tls {
                                resolver: tls_resolver,
                                options: tls_option,
                            };
                            match crate::tls::build_tls_config(tls) {
                                Ok(config) => {
                                    tls_acceptor = Some(tokio_rustls::TlsAcceptor::from(config));
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to build controller http listener TLS config: {}",
                                        e
                                    );
                                    break 'bind_http;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to resolve controller http listener TLS: {}",
                                e
                            );
                            break 'bind_http;
                        }
                    }
                }
                let bind_result = tokio::net::TcpListener::bind(addr).await;
                let listener = match bind_result {
                    Err(e) => {
                        tracing::error!(
                            "Failed to bind controller http listener on {}: {}",
                            addr,
                            e
                        );
                        break 'bind_http;
                    }
                    Ok(listener) => listener,
                };
                if let Some(tls) = tls_acceptor {
                    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener)
                        .and_then(move |stream| {
                            let tls = tls.clone();
                            async move {
                                let accept_result = tls.accept(stream).await;
                                match accept_result {
                                    Ok(tls_stream) => Ok(tls_stream),
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to accept TLS connection on controller http listener: {}",
                                            e
                                        );
                                        Err(e)
                                    }
                                }
                            }
                        });
                    tracing::info!("Controller https gRPC listening on {}", addr);
                    let span = tracing::info_span!("controller-https-listener", %addr);
                    join_set.spawn(
                        tonic::transport::Server::builder()
                            .add_service(grpc_server.clone())
                            .serve_with_incoming_shutdown(
                                incoming,
                                ct.child_token().cancelled_owned(),
                            )
                            .instrument(span),
                    );
                } else {
                    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);
                    tracing::info!("Controller http gRPC listening on {}", addr);
                    let span = tracing::info_span!("controller-http-listener", %addr);
                    join_set.spawn(
                        tonic::transport::Server::builder()
                            .add_service(grpc_server.clone())
                            .serve_with_incoming_shutdown(
                                incoming,
                                ct.child_token().cancelled_owned(),
                            )
                            .instrument(span),
                    );
                }
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
                tracing::info!("Controller uds gRPC listening on {}", path_display);
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
