use std::{borrow::Cow, sync::Arc};

use futures::{FutureExt, future::BoxFuture};
pub use switchboard_custom_config::{CustomConfig, Error as PayloadError, formats::PayloadObject};
use tcp::{SharedTcpService, TcpService};
use tokio::io::AsyncRead;
use udp::UdpService;

pub mod registry;
pub mod tcp;
pub mod udp;
pub mod utils;

pub use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct ServiceProviderMeta {
    pub version: String,
    pub author: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub tags: Vec<String>,
}

impl ServiceProviderMeta {
    pub fn from_env() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            author: env!("CARGO_PKG_AUTHORS")
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            description: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
            license: Some(env!("CARGO_PKG_LICENSE").to_string()),
            repository: Some(env!("CARGO_PKG_REPOSITORY").to_string()),
            homepage: Some(env!("CARGO_PKG_HOMEPAGE").to_string()),
            tags: Vec::new(),
        }
    }
}

impl Default for ServiceProviderMeta {
    fn default() -> Self {
        Self::from_env()
    }
}

pub trait TcpServiceProvider: Send + Sync + 'static {
    const NAME: &'static str;
    type Service: TcpService;
    type Error: std::error::Error + Send + Sync;
    fn meta(&self) -> ServiceProviderMeta {
        ServiceProviderMeta::from_env()
    }
    fn construct(
        &self,
        config: Option<CustomConfig>,
    ) -> impl Future<Output = Result<Self::Service, Self::Error>> + Send + '_;
}

pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxTcpServiceProvider = Box<dyn DynTcpServiceProvider>;
pub trait DynTcpServiceProvider: Send + Sync + 'static {
    fn meta(&self) -> ServiceProviderMeta;
    fn name(&self) -> Cow<'static, str>;
    fn construct(
        &self,
        config: Option<CustomConfig>,
    ) -> BoxFuture<'_, Result<SharedTcpService, BoxedError>>;
}

impl<T: TcpServiceProvider> DynTcpServiceProvider for T {
    fn meta(&self) -> ServiceProviderMeta {
        self.meta()
    }
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(T::NAME)
    }
    fn construct(
        &self,
        config: Option<CustomConfig>,
    ) -> BoxFuture<'_, Result<SharedTcpService, BoxedError>> {
        self.construct(config)
            .map(|result| {
                result
                    .map(|service| Arc::new(service) as SharedTcpService)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })
            .boxed()
    }
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
