mod config;
mod controller;
mod kernel;
mod state;

use axum::response::IntoResponse as _;
use http::HeaderValue;
use serde::Serialize;
use switchboard_kernel::KernelContext;
use tower_http::services::{ServeDir, ServeFile};

use crate::WebuiConfig;

#[derive(Clone)]
pub struct HttpState {
    pub kernel_context: KernelContext,
}

pub fn build_axum_router(http_state: HttpState, config: WebuiConfig) -> axum::Router<()> {
    let api_router = axum::Router::new()
        .nest("/kernel-api", kernel::router())
        .nest("/controller-api", controller::router());

    let Some(web_root) = config
        .enable_web_frontend
        .then_some(config.web_root.clone())
    else {
        return api_router.with_state(http_state);
    };

    let index_file = web_root.join("index.html");
    let static_service = ServeDir::new(web_root)
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new(index_file));

    api_router
        .fallback_service(static_service)
        .with_state(http_state)
}

fn result_to_json_response<T: Serialize, E: std::error::Error + 'static>(
    result: Result<T, E>,
) -> axum::response::Response {
    match result {
        Ok(data) => axum::Json(data).into_response(),
        Err(e) => {
            tracing::warn!("Internal server error: {}", e);
            let error_response = switchboard_model::error::ErrorStack::from_std(e);
            let mut response = axum::Json(error_response).into_response();
            response
                .headers_mut()
                .append("x-error-stack", HeaderValue::from_static(""));
            *response.status_mut() = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
            response
        }
    }
}
