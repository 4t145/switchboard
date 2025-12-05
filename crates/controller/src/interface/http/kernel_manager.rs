use std::collections::{BTreeMap, HashMap};

use axum::{Json, extract::State, response::Response};
use switchboard_model::{
    error::{ErrorStack, ResultObject},
    kernel::KernelConnectionAndState,
};

use crate::{interface::http::HttpState, kernel::KernelAddr};

pub async fn get_kernel_states(
    State(state): State<HttpState>,
) -> Json<BTreeMap<KernelAddr, KernelConnectionAndState>> {
    let kernel_manager = state.controller_context.kernel_manager.read().await;
    let states = kernel_manager.get_kernel_states().await;
    Json(states)
}

pub async fn update_config(
    State(state): State<HttpState>,
    Json(new_config): Json<switchboard_model::Config>,
) -> Json<Vec<(KernelAddr, ResultObject<()>)>> {
    let kernel_manager = state.controller_context.kernel_manager.read().await;
    let results = kernel_manager.update_config(new_config).await;
    let string_results = results
        .into_iter()
        .map(|(addr, result)| (addr, ResultObject::from(result)))
        .collect();
    Json(string_results)
}

pub async fn refresh_kernels(State(state): State<HttpState>) -> Response {
    super::result_to_json_response(state.controller_context.refresh_kernels().await)
}

pub async fn take_over_all_kernels(State(state): State<HttpState>) -> Response {
    super::result_to_json_response(state.controller_context.take_over_all_kernels().await)
}

pub async fn take_over_kernel(
    State(state): State<HttpState>,
    axum::extract::Path(addr): axum::extract::Path<KernelAddr>,
) -> Response {
    super::result_to_json_response(state.controller_context.take_over_kernel(addr).await)
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .route(
            "/kernels",
            axum::routing::get(get_kernel_states).put(update_config),
        )
        .route("/kernels/refresh", axum::routing::post(refresh_kernels))
        .route(
            "/kernels/take_over_all",
            axum::routing::post(take_over_all_kernels),
        )
        .route(
            "/kernels/take_over/{addr}",
            axum::routing::post(take_over_kernel),
        )
}