use std::sync::Arc;

use tokio::io::{self, AsyncRead, AsyncWrite};

use crate::{TcpService, tcp::AsyncStream};

#[derive(Debug, Clone)]
pub struct TlsService<S> {
    pub config: Arc<rustls::ServerConfig>,
    pub service: Arc<S>,
}
impl<Svc: TcpService + Send + Sync> TcpService for TlsService<Svc> {
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
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to accept TLS connection: {}", e),
                )
            })?;
        self.service.clone().serve(stream, peer, ct).await
    }
}
