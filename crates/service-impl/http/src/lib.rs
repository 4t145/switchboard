// pub mod config;
mod consts;
pub mod layer;
pub mod instance;
pub mod response;
// pub mod router;
pub mod service;
pub mod utils;
pub mod extension;
pub mod flow;
mod dynamic;
pub use consts::*;
mod export;
pub use dynamic::*;

use hyper::server::conn::{http1, http2};
use hyper_util::rt::{TokioExecutor, TokioIo};
// use instance::{InstanceId, orchestration::OrchestrationError, registry::InstanceRegistry};
use rustls::ServerConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use switchboard_service::TcpServiceProvider;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use utils::read_version;

use crate::flow::Flow;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HttpVersion {
    Http1,
    #[serde(alias = "h2")]
    Http2,
    #[default]
    Auto,
}

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
    #[instrument(skip_all, fields(peer = %peer))]
    async fn serve_http1(
        self,
        stream: impl switchboard_service::tcp::AsyncStream,
        peer: std::net::SocketAddr,
        ct: CancellationToken,
    ) -> std::io::Result<()> {
        let io = TokioIo::new(stream);
        let connection = http1::Builder::new()
            .serve_connection(io, self.service)
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

    #[instrument(skip_all, fields(peer = %peer))]
    async fn serve_http2(
        self,
        stream: impl switchboard_service::tcp::AsyncStream,
        peer: std::net::SocketAddr,
        ct: CancellationToken,
    ) -> std::io::Result<()> {
        let io = TokioIo::new(stream);
        let connection =
            http2::Builder::new(TokioExecutor::new()).serve_connection(io, self.service);
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
}

impl switchboard_service::tcp::TcpService for Http {
    async fn serve<S>(
        self: Arc<Self>,
        stream: S,
        peer: std::net::SocketAddr,
        ct: CancellationToken,
    ) -> std::io::Result<()>
    where
        S: switchboard_service::tcp::AsyncStream,
    {
        match self.version {
            HttpVersion::Http1 => self.as_ref().clone().serve_http1(stream, peer, ct).await,
            HttpVersion::Http2 => self.as_ref().clone().serve_http2(stream, peer, ct).await,
            HttpVersion::Auto => {
                let read_version = read_version(stream);
                let (version, rewind) = tokio::select! {
                    read_vesion = read_version => {
                        read_vesion?
                    }
                    _ = ct.cancelled() => {
                        tracing::debug!(%peer, "Auto version detection cancelled");
                        return Ok(());
                    }
                };
                tracing::debug!(%peer, "Detected HTTP version: {:?}", version);
                match version {
                    HttpVersion::Http1 => self.as_ref().clone().serve_http1(rewind, peer, ct).await,
                    HttpVersion::Http2 => self.as_ref().clone().serve_http2(rewind, peer, ct).await,
                    HttpVersion::Auto => {
                        unreachable!("Auto version should not be used here");
                    }
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpBuildError {
    // #[error("Failed to build HTTP service: {0}")]
    // Orchestration(#[from] OrchestrationError),

    // #[error("Failed to parse config: {0}")]
    // ParseError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize)]
pub struct HttpConfig {
    // pub objects: InstanceRegistry,
    // pub entrypoint: InstanceId,
}

pub struct HttpProvider;

impl TcpServiceProvider for HttpProvider {
    const NAME: &'static str = "http";
    type Service = Http;
    type Error = HttpBuildError;
    async fn construct(&self, config: Option<String>) -> Result<Self::Service, Self::Error> {
        todo!();
        // let config = config.unwrap_or_default();
        // let config = serde_json::from_str::<HttpConfig>(&config)?;
        // let class_registry = crate::instance::registry::ClassRegistry::globol()
        //     .read_owned()
        //     .await;
        // let class_registry = &class_registry;
        // let object_registry = config.objects;
        // let mut context = crate::instance::orchestration::OrchestrationContext::new(
        //     class_registry,
        //     &object_registry,
        // );
        // let mut orchestration = crate::instance::orchestration::Orchestration::default();
        // orchestration.rebuild_all_target(&mut context)?;
        // let service = orchestration.get_or_build_service(&config.entrypoint, &mut context)?;
        // Ok(Http {
        //     service,
        //     version: HttpVersion::default(),
        // })
    }
}
