use std::collections::BTreeMap;

use axum::{Json, extract::State, response::Response};
use switchboard_link_or_value::LinkOrValue;
use switchboard_model::{SerdeValue, kernel::KernelConnectionAndState};

use crate::{interface::http::HttpState, kernel::KernelAddr, link_resolver::Link};

pub async fn get_kernel_states(
    State(state): State<HttpState>,
) -> Json<BTreeMap<KernelAddr, KernelConnectionAndState>> {
    let kernel_manager = state.controller_context.kernel_manager.read().await;
    let states = kernel_manager.get_kernel_states().await;
    Json(states)
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum UpdateConfigRequest {
    NewConfig {
        new_config: LinkOrValue<Link, SerdeValue>,
    },
    Resolve {
        resolver: String,
        config: SerdeValue,
    },
}

pub async fn update_config(
    State(state): State<HttpState>,
    Json(request): Json<UpdateConfigRequest>,
) -> Response {
    let process = async move {
        let standard_config = match request {
            UpdateConfigRequest::NewConfig { new_config } => {
                let resolver = state.controller_context.clone().link_resolver();
                switchboard_model::resolve::file_style::fetch_config(new_config, &resolver).await?
            }
            UpdateConfigRequest::Resolve { resolver, config } => {
                let resolved_config = state
                    .controller_context
                    .resolve_config(&resolver, config)
                    .await?;
                let link_resolver = state.controller_context.clone().link_resolver();
                resolved_config.resolve_into_standard(&link_resolver).await?
            }
        };
        let results = state
            .controller_context
            .update_config(standard_config)
            .await;
        Ok::<_, crate::Error>(results)
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
