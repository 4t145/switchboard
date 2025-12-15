use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use switchboard_service::{
    BytesPayload, PayloadError, TcpServiceProvider,
    tcp::{AsyncStream, TcpService},
};

use tokio::io;
#[cfg(target_family = "unix")]
use tokio::net::UnixSocket;
#[derive(Debug, Clone)]
pub struct Uds {
    pub to: PathBuf,
}

impl TcpService for Uds {
    async fn serve<S>(
        self: Arc<Self>,
        stream: S,
        peer: SocketAddr,
        ct: tokio_util::sync::CancellationToken,
    ) -> io::Result<()>
    where
        S: AsyncStream,
    {
        tokio::select! {
            _ = ct.cancelled() => {
                Ok(())
            }
            result = forward_uds(stream, peer, &self.to) => result
        }
    }
}

#[cfg(target_family = "unix")]
async fn forward_uds<T: AsyncStream>(
    mut inbound: T,
    from: SocketAddr,
    to: &Path,
) -> io::Result<()> {
    let mut out = UnixSocket::new_stream()?.connect(to).await?;
    tracing::debug!(%from, ?to, "port forwarding");
    io::copy_bidirectional(&mut inbound, &mut out).await?;
    Ok(())
}

#[cfg(not(target_family = "unix"))]
async fn forward_uds<T: AsyncStream>(
    mut inbound: T,
    from: SocketAddr,
    to: &Path,
) -> io::Result<()> {
    tracing::warn!(%from, ?to, "UDS is not supported on this platform");
    Err(io::Error::other("UDS is only supported on Unix platforms"))
}

pub struct UdsProvider;
impl TcpServiceProvider for UdsProvider {
    const NAME: &'static str = "uds";
    type Service = Uds;
    type Error = PayloadError;

    async fn construct(&self, config: Option<BytesPayload>) -> Result<Self::Service, Self::Error> {
        let config: String = config.unwrap_or_default().decode()?;
        let to = PathBuf::from_str(&config).expect("infallible error");
        Ok(Uds { to })
    }
}
