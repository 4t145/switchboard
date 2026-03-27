use axum::{Json, extract::State, response::Response};
use switchboard_model::ServiceConfig;

use crate::http::HttpState;

#[derive(Debug, serde::Deserialize)]
pub struct UpdateConfigRequest {
    pub config: ServiceConfig,
}

#[derive(Debug, serde::Deserialize)]
pub struct PrepareConfigRequest {
    pub transaction_id: String,
    pub expected_version: String,
    pub config: ServiceConfig,
    pub ttl_secs: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CommitConfigRequest {
    pub transaction_id: String,
    pub expected_version: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AbortConfigRequest {
    pub transaction_id: String,
}

pub async fn update_config(
    State(state): State<HttpState>,
    Json(request): Json<UpdateConfigRequest>,
) -> Response {
    super::result_to_json_response(state.kernel_context.update_config(request.config).await)
}

pub async fn prepare_config(
    State(state): State<HttpState>,
    Json(request): Json<PrepareConfigRequest>,
) -> Response {
    super::result_to_json_response(
        state
            .kernel_context
            .prepare_config(
                request.transaction_id,
                request.config,
                request.expected_version,
                request.ttl_secs,
            )
            .await,
    )
}

pub async fn commit_config(
    State(state): State<HttpState>,
    Json(request): Json<CommitConfigRequest>,
) -> Response {
    super::result_to_json_response(
        state
            .kernel_context
            .commit_config(&request.transaction_id, &request.expected_version)
            .await,
    )
}

pub async fn abort_config(
    State(state): State<HttpState>,
    Json(request): Json<AbortConfigRequest>,
) -> Response {
    super::result_to_json_response(
        state
            .kernel_context
            .abort_config(&request.transaction_id)
            .await,
    )
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .route("/update_config", axum::routing::post(update_config))
        .route("/prepare_config", axum::routing::post(prepare_config))
        .route("/commit_config", axum::routing::post(commit_config))
        .route("/abort_config", axum::routing::post(abort_config))
}
