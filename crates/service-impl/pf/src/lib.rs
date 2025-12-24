use std::{net::SocketAddr, pin::Pin, sync::Arc};

use switchboard_service::{
    SerdeValue, SerdeValueError, TcpServiceProvider,
    tcp::{AsyncStream, TcpService},
};
use tokio::{io, net::TcpStream};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct PortForward {
    pub to: SocketAddr,
}

impl PortForward {
    async fn serve_inner<S>(
        self: Arc<Self>,
        stream: S,
        ct: CancellationToken,
        peer: SocketAddr,
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

impl TcpService for PortForward {
    fn name(&self) -> &str {
        "port-forward"
    }
    fn serve(
        self: Arc<Self>,
        accepted: switchboard_service::tcp::TcpAccepted,
    ) -> Pin<Box<dyn Future<Output = io::Result<()>> + 'static + Send>> {
        Box::pin(self.serve_inner(
            accepted.stream,
            accepted.context.ct,
            accepted.context.peer_addr,
        ))
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
    type Error = SerdeValueError;

    async fn construct(&self, config: Option<SerdeValue>) -> Result<Self::Service, Self::Error> {
        let to = config.unwrap_or_default().deserialize_into()?;
        Ok(PortForward { to })
    }
}
