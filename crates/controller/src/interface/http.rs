pub mod kernel_manager;

use std::net::SocketAddr;

use axum::response::IntoResponse as _;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::ControllerContext;
#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct HttpInterfaceConfig {
    pub bind: SocketAddr,
}

impl Default for HttpInterfaceConfig {
    fn default() -> Self {
        HttpInterfaceConfig {
            bind: SocketAddr::from(([0, 0, 0, 0], 8056)),
        }
    }
}

pub struct HttpInterface {
    pub config: HttpInterfaceConfig,
    pub ct: CancellationToken,
    pub handle: tokio::task::JoinHandle<()>,
}

#[derive(Clone)]
pub struct HttpState {
    pub controller_context: ControllerContext,
}

impl ControllerContext {
    pub fn http_state(&self) -> HttpState {
        HttpState {
            controller_context: self.clone(),
        }
    }
    pub fn build_axum_router(&self) -> axum::Router<()> {
        axum::Router::new()
            .nest(
                "/api",
                axum::Router::new().nest("/kernel_manager", kernel_manager::router()),
            )
            .with_state(self.http_state())
    }
    pub async fn start_up_http_interface(
        &self,
        config: HttpInterfaceConfig,
    ) -> crate::Result<HttpInterface> {
        let ct = tokio_util::sync::CancellationToken::new();
        let router = self.build_axum_router();
        let bind_addr = config.bind;
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .map_err(crate::Error::StartupHttpInterfaceError)?;
        tracing::info!("HTTP interface listening on {}", bind_addr);
        let server = axum::serve(listener, router);
        let graceful = server.with_graceful_shutdown(ct.clone().cancelled_owned());
        let handle = tokio::spawn(async move {
            if let Err(e) = graceful.await {
                tracing::error!("HTTP server error: {}", e);
            }
        });
        Ok(HttpInterface { config, ct, handle })
    }
}

/// Converts a Result into an axum Response with JSON body.
///
/// if the result is Ok, the response will contain the serialized data.
///
/// if the result is Err, the response will contain the serialized ErrorStack and a 500 status code.
fn result_to_json_response<T: Serialize, E: std::error::Error + 'static>(
    result: Result<T, E>,
) -> axum::response::Response {
    match result {
        Ok(data) => axum::Json(data).into_response(),
        Err(e) => {
            let error_response = switchboard_model::error::ErrorStack::from_std(e);
            let mut response = axum::Json(error_response).into_response();
            *response.status_mut() = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
            response
        }
    }
}
