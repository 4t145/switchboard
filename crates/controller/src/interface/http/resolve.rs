use axum::{Json, extract::State, response::Response};
use switchboard_custom_config::SerdeValue;
use switchboard_model::{ServiceConfig, error::ResultObject};

use crate::{interface::http::HttpState, storage::StorageObjectDescriptor};
#[derive(Debug, serde::Deserialize)]
pub struct ResolveServiceConfigRequest {
    pub resolver: String,
    #[serde(default)]
    pub save_as: Option<String>,
    pub config: SerdeValue,
}
#[derive(Debug, serde::Serialize)]

pub struct ResolveServiceConfigResponse {
    pub descriptor: Option<StorageObjectDescriptor>,
    pub config: ServiceConfig,
}

pub async fn resolve_service_config(
    State(state): State<HttpState>,
    Json(request): Json<ResolveServiceConfigRequest>,
) -> Response {
    let process = async {
        let config = state
            .controller_context
            .resolve_config(&request.resolver, request.config)
            .await?;
        let descriptor = if let Some(save_as) = request.save_as {
            Some(
                state
                    .controller_context
                    .save_known_object(&save_as, config.clone())
                    .await?,
            )
        } else {
            None
        };
        crate::Result::Ok(ResolveServiceConfigResponse { config, descriptor })
    };
    super::result_to_json_response(process.await)
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new().route(
        "/service_config",
        axum::routing::post(resolve_service_config),
    )
}
