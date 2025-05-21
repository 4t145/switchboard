
pub mod config;
pub mod layer;
pub mod object;
pub mod response;
pub mod router;
pub mod service;
pub mod utils;

use hyper::server::conn::{http1, http2};
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::ServerConfig;
use serde::{Deserialize, Serialize};
use service::dynamic::SharedService;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use utils::read_version;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpVersion {
    Http1,
    #[serde(alias = "h2")]
    Http2,
    Auto,
}

pub enum Tls {
    Tls { config: Arc<ServerConfig> },
    NoTls,
    Auto { config: Arc<ServerConfig> },
}

#[derive(Clone)]
pub struct Http {
    service: SharedService,
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
                    std::io::Error::new(std::io::ErrorKind::Other, e)
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
                    std::io::Error::new(std::io::ErrorKind::Other, e)
                })?;
            }
        }
        Ok(())
    }
}

impl switchboard_service::tcp::TcpService for Http {
    async fn serve<S>(
        self,
        stream: S,
        peer: std::net::SocketAddr,
        ct: CancellationToken,
    ) -> std::io::Result<()>
    where
        S: switchboard_service::tcp::AsyncStream,
    {
        match self.version {
            HttpVersion::Http1 => self.serve_http1(stream, peer, ct).await,
            HttpVersion::Http2 => self.serve_http2(stream, peer, ct).await,
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
                    HttpVersion::Http1 => self.serve_http1(rewind, peer, ct).await,
                    HttpVersion::Http2 => self.serve_http2(rewind, peer, ct).await,
                    HttpVersion::Auto => {
                        unreachable!("Auto version should not be used here");
                    }
                }
            }
        }
    }
}
