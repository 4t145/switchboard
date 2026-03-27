use std::{path::PathBuf, sync::Arc};

mod error;
pub mod http;

pub use error::{Error, Result};
use switchboard_kernel::KernelContext;
use switchboard_service::{
    SerdeValue, SerdeValueError, TcpServiceProvider,
    tcp::{TcpAccepted, TcpService},
};

use hyper::server::conn::http1;
use hyper_util::{rt::TokioIo, service::TowerToHyperService};

const DEFAULT_WEB_ROOT: &str = "./ui/web/";

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WebuiConfig {
    pub enable_web_frontend: bool,
    pub web_root: PathBuf,
}

impl Default for WebuiConfig {
    fn default() -> Self {
        Self {
            enable_web_frontend: true,
            web_root: PathBuf::from(DEFAULT_WEB_ROOT),
        }
    }
}

#[derive(Clone)]
pub struct Webui {
    router: axum::Router<()>,
}

impl Webui {
    async fn serve_inner(self: Arc<Self>, accepted: TcpAccepted) -> std::io::Result<()> {
        let accepted = accepted.maybe_tls_terminate().await?;
        let peer_addr = accepted.context.peer_addr;
        let ct = accepted.context.ct;
        let io = TokioIo::new(accepted.stream);
        let service = TowerToHyperService::new(self.router.clone());
        let connection = http1::Builder::new()
            .serve_connection(io, service)
            .with_upgrades();

        tokio::select! {
            _ = ct.cancelled() => {
                tracing::debug!(%peer_addr, "webui connection cancelled");
                Ok(())
            }
            result = connection => {
                result.map_err(|e| {
                    tracing::error!(%peer_addr, "webui connection error: {}", e);
                    std::io::Error::other(e)
                })
            }
        }
    }
}

impl TcpService for Webui {
    fn name(&self) -> &str {
        "webui"
    }

    fn serve(
        self: Arc<Self>,
        accepted: TcpAccepted,
    ) -> futures::future::BoxFuture<'static, std::io::Result<()>> {
        Box::pin(self.serve_inner(accepted))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebuiBuildError {
    #[error("failed to decode config: {0}")]
    PayloadDecodeError(#[from] SerdeValueError),
}

#[derive(Clone)]
pub struct WebInterfaceProvider {
    pub kernel_context: KernelContext,
}

impl TcpServiceProvider for WebInterfaceProvider {
    const NAME: &'static str = "webui";
    type Service = Webui;
    type Error = WebuiBuildError;

    async fn construct(
        &self,
        config: Option<SerdeValue>,
    ) -> std::result::Result<Self::Service, Self::Error> {
        let config: WebuiConfig = config.unwrap_or_default().deserialize_into()?;
        let router = http::build_axum_router(
            http::HttpState {
                kernel_context: self.kernel_context.clone(),
            },
            config,
        );
        Ok(Webui { router })
    }
}
