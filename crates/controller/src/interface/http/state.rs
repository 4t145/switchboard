use std::collections::BTreeMap;

use axum::{Json, extract::State, response::Response};
use switchboard_link_or_value::LinkOrValue;
use switchboard_model::{
    HumanReadableServiceConfig, SerdeValue, error::ResultObject, kernel::KernelConnectionAndState,
    resolve::file_style::ResolveConfigFileError,
};

use crate::{
    interface::http::HttpState,
    kernel::{KernelAddr, KernelGrpcConnectionError},
    link_resolver::Link,
};

pub async fn get_current_config(
    State(state): State<HttpState>,
) -> Json<Option<HumanReadableServiceConfig<Link>>> {
    let config = state
        .controller_context
        .current_config
        .read()
        .await
        .as_ref()
        .map(|config| {
            let human_readable_config =
                <HumanReadableServiceConfig<Link>>::from_standard(config.clone());
            human_readable_config
        });
    Json(config)
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new().route("/current_config", axum::routing::get(get_current_config))
}
