use std::collections::BTreeMap;

use axum::{Json, extract::State, response::Response};
use switchboard_link_or_value::LinkOrValue;
use switchboard_model::{
    SerdeValue, error::ResultObject, kernel::KernelConnectionAndState,
    resolve::file_style::ResolveConfigFileError,
};

use crate::{
    interface::http::HttpState,
    kernel::{KernelAddr, KernelGrpcConnectionError},
    link_resolver::Link,
};

pub async fn get_kernel_states(
    State(state): State<HttpState>,
) -> Json<BTreeMap<KernelAddr, KernelConnectionAndState>> {
    let kernel_manager = state.controller_context.kernel_manager.read().await;
    let states = kernel_manager.get_kernel_states().await;
    Json(states)
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateConfigRequest {
    pub new_config: LinkOrValue<Link, SerdeValue>,
}

pub async fn update_config(
    State(state): State<HttpState>,
    Json(request): Json<UpdateConfigRequest>,
) -> Response {
    let process = async move {
        let resolver = state.controller_context.clone().link_resolver();
        let standard_config =
            switchboard_model::resolve::file_style::fetch_config(request.new_config, &resolver)
                .await?;
        let results = state
            .controller_context
            .update_config(standard_config)
            .await;
        let string_results: Vec<(KernelAddr, ResultObject<()>)> = results
            .into_iter()
            .map(|(addr, result)| (addr, ResultObject::from(result)))
            .collect();
        Ok::<_, ResolveConfigFileError>(string_results)
    };
    super::result_to_json_response(process.await)
}

pub async fn refresh_kernels(State(state): State<HttpState>) -> Response {
    super::result_to_json_response(state.controller_context.refresh_kernels().await)
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .route(
            "/kernels",
            axum::routing::get(get_kernel_states).put(update_config),
        )
        .route("/refresh", axum::routing::post(refresh_kernels))
}
