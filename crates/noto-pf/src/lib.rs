use std::net::SocketAddr;

use noto_tcp::{AsyncStream, TcpService};
use tokio::{io, net::TcpStream};

#[derive(Debug, Clone)]
pub struct PortForward {
    pub to: SocketAddr,
}

impl TcpService for PortForward {
    async fn serve<S>(
        self,
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
