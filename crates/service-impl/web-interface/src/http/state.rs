use axum::{Json, extract::State};
use switchboard_model::kernel::KernelState;

use crate::http::HttpState;

pub async fn get_current_state(State(state): State<HttpState>) -> Json<KernelState> {
    Json(state.kernel_context.get_state())
}

pub async fn get_current_config(
    State(state): State<HttpState>,
) -> axum::response::Response {
    super::result_to_json_response(state.kernel_context.fetch_config_locally().await)
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .route("/current_state", axum::routing::get(get_current_state))
        .route("/current_config", axum::routing::get(get_current_config))
}
