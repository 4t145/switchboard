use std::sync::Arc;

use tokio::io::{self, AsyncRead, AsyncWrite};

use crate::{tcp::AsyncStream, TcpService};

#[derive(Debug, Clone)]
pub struct TlsService<S> {
    pub config: Arc<rustls::ServerConfig>,
    pub service: S,
}
impl<Svc: TcpService + Send + Sync> TcpService for TlsService<Svc> {
    async fn serve<S>(
        self,
        stream: S,
        peer: std::net::SocketAddr,
        ct: tokio_util::sync::CancellationToken,
    ) -> io::Result<()>
    where
        S: AsyncStream,
    {
        let stream = tokio_rustls::TlsAcceptor::from(self.config.clone())
            .accept(stream)
            .await
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to accept TLS connection: {}", e),
                )
            })?;
        self.service.serve(stream, peer, ct).await
    }
}
