use std::net::SocketAddr;
use tokio::{io, net::UdpSocket};
use tokio_util::sync::CancellationToken;

pub trait UdpService: Send + Clone + 'static {
    fn serve(
        self,
        socket: UdpSocket,
        ct: CancellationToken,
    ) -> impl Future<Output = std::io::Result<()>> + Send + 'static;
}

pub trait UdpServiceExt: UdpService {
    fn bind(&self, addr: SocketAddr) -> impl Future<Output = io::Result<RunningUdpService>>;
    fn listen(self, socket: UdpSocket, ct: CancellationToken) -> impl Future<Output = ()>;
}

impl<S: UdpService> UdpServiceExt for S {
    async fn bind(&self, addr: SocketAddr) -> io::Result<RunningUdpService> {
        let socket = UdpSocket::bind(addr).await?;
        tracing::info!(%addr, "Listening on UDP");
        let ct = CancellationToken::new();
        let join_handle = tokio::spawn(self.clone().listen(socket, ct.child_token()));
        Ok(RunningUdpService {
            bind: addr,
            ct,
            join_handle,
        })
    }

    async fn listen(self, socket: UdpSocket, ct: CancellationToken) {
        todo!()
    }
}

pub struct RunningUdpService {
    bind: SocketAddr,
    ct: CancellationToken,
    join_handle: tokio::task::JoinHandle<()>,
}

impl RunningUdpService {
    pub fn bind(&self) -> SocketAddr {
        self.bind
    }
    pub async fn wait(self) -> Result<(), tokio::task::JoinError> {
        self.join_handle.await
    }
    pub async fn cancel(self) -> Result<(), tokio::task::JoinError> {
        self.ct.cancel();
        self.join_handle.await
    }
}
