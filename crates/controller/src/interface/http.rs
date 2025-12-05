pub mod kernel_manager;

use std::net::SocketAddr;

use axum::response::IntoResponse as _;
use serde::Serialize;
use tokio_util::sync::CancellationToken;

use crate::ControllerContext;

pub struct HttpInterfaceConfig {
    pub bind: SocketAddr,
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
            .nest("/kernel_manager", kernel_manager::router())
            .with_state(self.http_state())
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
