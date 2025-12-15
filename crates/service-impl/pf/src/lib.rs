use std::{net::SocketAddr, sync::Arc};

use switchboard_service::{
    BytesPayload, PayloadError, TcpServiceProvider,
    tcp::{AsyncStream, TcpService},
};
use tokio::{io, net::TcpStream};

#[derive(Debug, Clone)]
pub struct PortForward {
    pub to: SocketAddr,
}

impl TcpService for PortForward {
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
            result = forward_tcp(stream, peer, self.to) => result
        }
    }
}

async fn forward_tcp<T: AsyncStream>(
    mut inbound: T,
    from: SocketAddr,
    to: SocketAddr,
) -> io::Result<()> {
    let mut out = TcpStream::connect(to).await?;
    tracing::debug!(%from, %to, "port forwarding");
    tokio::io::copy_bidirectional(&mut inbound, &mut out).await?;
    Ok(())
}

pub struct PortForwardProvider;
impl TcpServiceProvider for PortForwardProvider {
    const NAME: &'static str = "pf";
    type Service = PortForward;
    type Error = PayloadError;

    async fn construct(&self, config: Option<BytesPayload>) -> Result<Self::Service, Self::Error> {
        let to = config.unwrap_or_default().decode()?;
        Ok(PortForward { to })
    }
}
