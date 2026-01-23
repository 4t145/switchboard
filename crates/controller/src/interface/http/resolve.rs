use axum::{
    Json,
    extract::{Query, State},
    response::Response,
};
use switchboard_model::{HumanReadableServiceConfig, SerdeValue};
use switchboard_link_or_value::Writer;

use crate::{interface::http::HttpState, link_resolver::Link, storage::StorageObjectDescriptor};
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
    pub config: HumanReadableServiceConfig<Link>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ResolveObjectQuery {
    pub link: Link,
}

#[derive(Debug, serde::Deserialize)]
pub struct SaveToLinkRequest {
    pub link: String,
    pub value: SerdeValue,
    pub data_type: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SaveToLinkResponse {
    pub link: String,
}

pub async fn resolve_value(
    State(state): State<HttpState>,
    Query(query): Query<ResolveObjectQuery>,
) -> Response {
    let process = async {
        let object: SerdeValue = state
            .controller_context
            .link_resolver()
            .resolve_link_to_value(query.link)
            .await?;
        crate::Result::Ok(object)
    };
    super::result_to_json_response(process.await)
}

pub async fn resolve_string(
    State(state): State<HttpState>,
    Query(query): Query<ResolveObjectQuery>,
) -> Response {
    let process = async {
        let string: String = state
            .controller_context
            .link_resolver()
            .resolve_link_to_string(query.link)
            .await?;
        crate::Result::Ok(string)
    };
    super::result_to_plaintext_response(process.await)
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

pub async fn save_to_link(
    State(state): State<HttpState>,
    Json(request): Json<SaveToLinkRequest>,
) -> Response {
    let process = async {
        let link: Link = request.link.parse()
            .map_err(crate::link_resolver::LinkResolveError::from)?;
        
        let updated_link = match link {
            Link::FilePath(path) => {
                // Save to file using FileResolver
                state.controller_context
                    .link_resolver()
                    .file_resolver()
                    .write(path, request.value)
                    .await
                    .map_err(crate::link_resolver::LinkResolveError::from)?;
                request.link  // File link remains unchanged
            },
            Link::Storage(descriptor) => {
                // Save to storage and get new descriptor
                let new_descriptor = state.controller_context
                    .storage()
                    .save_object(&descriptor.id, &request.data_type, request.value)
                    .await?;
                // Return updated link with new revision
                format!("storage://{}#{}", new_descriptor.id, new_descriptor.revision)
            },
            Link::Http(_uri) => {
                // HTTP PUT not yet supported
                return Err(crate::link_resolver::LinkResolveError::NoImplementation { 
                    link: Link::Http(_uri) 
                }.into());
            }
        };
        
        crate::Result::Ok(SaveToLinkResponse {
            link: updated_link,
        })
    };
    super::result_to_json_response(process.await)
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .route(
            "/service_config",
            axum::routing::post(resolve_service_config),
        )
        .route("/value", axum::routing::get(resolve_value))
        .route("/string", axum::routing::get(resolve_string))
        .route("/save_to_link", axum::routing::post(save_to_link))
}
