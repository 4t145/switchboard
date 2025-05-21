use std::borrow::Cow;

use futures::future::BoxFuture;
use tcp::{DynTcpService, TcpService};
use tokio::io::AsyncRead;
use udp::UdpService;

pub mod tcp;
pub mod udp;
pub mod utils;

pub trait TcpServiceProvider {
    const NAME: &'static str;
    type Service: TcpService;
    type Error: std::error::Error + Send;
    fn provide(
        &self,
        config: impl AsyncRead + Send,
    ) -> impl Future<Output = Result<Self::Service, Self::Error>>;
}

pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxTcpServiceProvider = Box<dyn DynTcpServiceProvider>;
pub trait DynTcpServiceProvider {
    fn name(&self) -> Cow<'static, str>;
    fn provide(
        &self,
        config: impl AsyncRead + Send,
    ) -> BoxFuture<'static, Result<Box<dyn DynTcpService>, BoxedError>>;
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
