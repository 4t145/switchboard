pub mod config;
mod consts;
mod dynamic;
pub mod extension;
pub mod flow;
pub mod instance;
pub mod response;
pub mod utils;
pub use consts::*;
pub use dynamic::*;

use hyper::server::conn::{http1, http2};
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::ServerConfig;
use std::{ops::Deref, sync::Arc};
use switchboard_model::services::http::HttpVersion;
use switchboard_service::{
    SerdeValue, SerdeValueError, TcpServiceProvider, tcp::{TcpAccepted, TcpConnectionContext}
};
use tokio_util::sync::CancellationToken;
use utils::read_version;

use crate::{
    flow::{ConnectionInfo, Flow, FlowWithConnectionInfo, build::FlowBuildError},
    instance::class::registry::ClassRegistry,
};

pub enum Tls {
    Tls { config: Arc<ServerConfig> },
    NoTls,
    Auto { config: Arc<ServerConfig> },
}

#[derive(Clone)]
pub struct Http {
    service: Flow,
    version: HttpVersion,
}

impl Http {
    async fn serve_http1(
        self,
        stream: impl switchboard_service::tcp::AsyncStream,
        mut connection_info: ConnectionInfo,
        ct: CancellationToken,
    ) -> std::io::Result<()> {
        connection_info.http_version = http::Version::HTTP_11;
        let io = TokioIo::new(stream);
        let peer = connection_info.peer_addr;
        let connection = http1::Builder::new()
            .serve_connection(
                io,
                FlowWithConnectionInfo {
                    flow: self.service,
                    connection_info,
                },
            )
            .with_upgrades();
        tokio::select! {
            _ = ct.cancelled() => {
                tracing::debug!(%peer, "connection cancelled");
                return Ok(());
            }
            result = connection => {
                result.map_err(|e| {
                    tracing::error!(%peer, "Error serving connection: {}", e);
                    std::io::Error::other(e)
                })?;
            }
        }
        Ok(())
    }

    async fn serve_http2(
        self,
        stream: impl switchboard_service::tcp::AsyncStream,
        mut connection_info: ConnectionInfo,
        ct: CancellationToken,
    ) -> std::io::Result<()> {
        connection_info.http_version = http::Version::HTTP_2;
        let io = TokioIo::new(stream);
        let peer = connection_info.peer_addr;
        let connection = http2::Builder::new(TokioExecutor::new()).serve_connection(
            io,
            FlowWithConnectionInfo {
                flow: self.service,
                connection_info,
            },
        );
        tokio::select! {
            _ = ct.cancelled() => {
                tracing::debug!(%peer, "HTTP/2 connection cancelled");
                return Ok(());
            }
            result = connection => {
                result.map_err(|e| {
                    tracing::error!(%peer, "Error serving HTTP/2 connection: {}", e);
                    std::io::Error::other(e)
                })?;
            }
        }
        Ok(())
    }

    async fn serve_inner(self: Arc<Self>, accepted: TcpAccepted) -> std::io::Result<()> {
        let accepted = accepted.maybe_tls().await?;
        let stream = accepted.stream;
        let is_tls = stream.is_tls();
        let TcpConnectionContext { peer_addr, ct, .. } = accepted.context;
        let connection_info = ConnectionInfo {
            peer_addr,
            http_version: http::Version::HTTP_11,
            is_tls,
        };
        match self.version {
            HttpVersion::Http1 => {
                self.as_ref()
                    .clone()
                    .serve_http1(stream, connection_info, ct)
                    .await
            }
            HttpVersion::Http2 => {
                self.as_ref()
                    .clone()
                    .serve_http2(stream, connection_info, ct)
                    .await
            }
            HttpVersion::Auto => {
                let read_version = read_version(stream);
                let (version, rewind) = tokio::select! {
                    read_version = read_version => {
                        read_version?
                    }
                    _ = ct.cancelled() => {
                        tracing::debug!(%peer_addr, "Auto version detection cancelled");
                        return Ok(());
                    }
                };
                tracing::trace!(%peer_addr, "Detected HTTP version: {:?}", version);
                match version {
                    HttpVersion::Http1 => {
                        self.as_ref()
                            .clone()
                            .serve_http1(rewind, connection_info, ct)
                            .await
                    }
                    HttpVersion::Http2 => {
                        self.as_ref()
                            .clone()
                            .serve_http2(rewind, connection_info, ct)
                            .await
                    }
                    HttpVersion::Auto => {
                        unreachable!("Auto version should not be used here");
                    }
                }
            }
        }
    }
}

impl switchboard_service::tcp::TcpService for Http {
    fn name(&self) -> &str {
        "http"
    }
    fn serve(
        self: Arc<Self>,
        accepted: TcpAccepted,
    ) -> futures::future::BoxFuture<'static, std::io::Result<()>> {
        Box::pin(self.serve_inner(accepted))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpBuildError {
    #[error("Failed to decode config: {0}")]
    PayloadDecodeError(#[from] SerdeValueError),

    #[error("Failed to build flow: {0}")]
    FlowBuildError(#[from] FlowBuildError),
}

pub struct HttpProvider;

impl TcpServiceProvider for HttpProvider {
    const NAME: &'static str = "http";
    type Service = Http;
    type Error = HttpBuildError;
    async fn construct(&self, config: Option<SerdeValue>) -> Result<Self::Service, Self::Error> {
        let config: config::Config = config.unwrap_or_default().deserialize_into()?;
        let class_registry = ClassRegistry::global();
        let flow = Flow::build(
            config.flow,
            class_registry.read_owned().await.deref(),
        )?;
        let service = Http {
            service: flow,
            version: config.server.version,
        };
        Ok(service)
    }
}
