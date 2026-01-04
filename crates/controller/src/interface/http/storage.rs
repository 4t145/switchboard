use axum::{
    Json,
    extract::{Query, State}, response::Response,
};
use switchboard_custom_config::SerdeValue;
use switchboard_model::{
    FlattenPageQueryWithFilter, PagedResult, ServiceConfig, error::ResultObject,
};

use crate::{
    interface::http::HttpState,
    storage::{ListObjectQuery, ObjectFilter, StorageObjectDescriptor, StorageObjectWithoutData},
};

#[derive(Debug, serde::Deserialize)]
pub struct SaveStorageObjectRequest {
    pub id: String,
    pub date_type: String,
    pub data: SerdeValue,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum BatchDeleteStorageObjectRequest {
    ByDescriptors {
        descriptors: Vec<StorageObjectDescriptor>,
    },
    ById {
        id: String,
    },
}

pub async fn get(
    State(state): State<HttpState>,
    Query(params): Query<StorageObjectDescriptor>,
) -> Response  {
    let process = async {
        let obj = state
            .controller_context
            .storage()
            .get_object(&params)
            .await?;
        crate::Result::Ok(obj)
    };
    super::result_to_json_response(process.await)
}

pub async fn delete(
    State(state): State<HttpState>,
    Query(params): Query<StorageObjectDescriptor>,
) -> Response  {
    let process = async {
        state
            .controller_context
            .storage()
            .delete_object(&params)
            .await?;
        crate::Result::Ok(())
    };
    super::result_to_json_response(process.await)
}

pub async fn batch_delete(
    State(state): State<HttpState>,
    Json(request): Json<BatchDeleteStorageObjectRequest>,
) -> Response  {
    let process = async {
        match request {
            BatchDeleteStorageObjectRequest::ByDescriptors { descriptors } => {
                state
                    .controller_context
                    .storage()
                    .batch_delete_objects(descriptors)
                    .await?;
            }
            BatchDeleteStorageObjectRequest::ById { id } => {
                state
                    .controller_context
                    .storage()
                    .delete_all_objects_by_id(&id)
                    .await?;
            }
        }
        crate::Result::Ok(())
    };
    super::result_to_json_response(process.await)
}

pub async fn save(
    State(state): State<HttpState>,
    Json(request): Json<SaveStorageObjectRequest>,
) -> Response {
    let process = async {
        let descriptor = state
            .controller_context
            .storage()
            .save_object(&request.id, &request.date_type, request.data)
            .await?;
        crate::Result::Ok(descriptor)
    };
    super::result_to_json_response(process.await)
}

pub async fn list(
    State(state): State<HttpState>,
    Query(params): Query<FlattenPageQueryWithFilter<ObjectFilter>>,
) -> Response  {
    let (page, filter) = params.into_parts();
    let process = async {
        let list = state
            .controller_context
            .storage()
            .list_objects(ListObjectQuery { page, filter })
            .await?;
        crate::Result::Ok(list)
    };
    super::result_to_json_response(process.await)
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .route("/object", axum::routing::get(get).delete(delete).post(save))
        .route("/objects", axum::routing::get(list).delete(batch_delete))
}
