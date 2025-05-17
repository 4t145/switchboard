use tcp::TcpService;
use tokio::io::AsyncRead;
use udp::UdpService;

pub mod tcp;
pub mod udp;

pub trait TcpServiceProvider {
    const NAME: &'static str;
    type Service: TcpService;
    type Error: std::error::Error + Send;
    fn provide(
        &self,
        config: impl AsyncRead + Send,
    ) -> impl Future<Output = Result<Self::Service, Self::Error>>;
}

pub trait UdpServiceProvider {
    const NAME: &'static str;

    type Service: UdpService;
    type Error: std::error::Error + Send;
    fn provide(
        &self,
        config: impl AsyncRead + Send,
    ) -> impl Future<Output = Result<Self::Service, Self::Error>>;
}
