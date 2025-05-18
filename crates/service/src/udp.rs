use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

pub trait UdpService: Send + Clone + 'static {
    fn serve(
        self,
        socket: UdpSocket,
        ct: CancellationToken,
    ) -> impl Future<Output = std::io::Result<()>> + Send + 'static;
}
