use axum::{Json, extract::State};

use switchboard_model::HumanReadableServiceConfig;

use crate::{interface::http::HttpState, link_resolver::Link};

pub async fn get_current_config(
    State(state): State<HttpState>,
) -> Json<Option<HumanReadableServiceConfig<Link>>> {
    let config = state
        .controller_context
        .current_config
        .read()
        .await
        .as_ref()
        .map(|config| <HumanReadableServiceConfig<Link>>::from_standard(config.clone()));
    Json(config)
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new().route("/current_config", axum::routing::get(get_current_config))
}
