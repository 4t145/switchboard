use std::sync::Arc;

use tokio::io;

use crate::{TcpService, tcp::AsyncStream};

use super::DynTcpService;

#[derive(Debug, Clone)]
pub struct TlsService<S: ?Sized = dyn DynTcpService> {
    pub config: Arc<rustls::ServerConfig>,
    pub service: Arc<S>,
}

impl<Svc: TcpService + Send + Sync + ?Sized> TcpService for TlsService<Svc> {
    async fn serve<S>(
        self: Arc<Self>,
        stream: S,
        peer: std::net::SocketAddr,
        ct: tokio_util::sync::CancellationToken,
    ) -> io::Result<()>
    where
        S: AsyncStream,
    {
        let config = self.config.clone();
        let stream = tokio_rustls::TlsAcceptor::from(config)
            .accept(stream)
            .await
            .map_err(|e| io::Error::other(format!("Failed to accept TLS connection: {}", e)))?;
        self.service.clone().serve(stream, peer, ct).await
    }
}
