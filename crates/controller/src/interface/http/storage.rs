use axum::{
    Json,
    extract::{Query, State},
    response::Response,
};
use chrono::{DateTime, Utc};
use switchboard_custom_config::SerdeValue;
use switchboard_model::{
    FlattenPageQueryWithFilter, PagedList, ServiceConfig, error::ResultObject,
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
) -> Response {
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
) -> Response {
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
) -> Response {
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ObjectFilterQuery {
    pub data_type: Option<String>,
    pub id: Option<String>,
    pub revision: Option<String>,
    pub latest_only: Option<BooleanLit>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum BooleanLit {
    True,
    False,
}

impl From<BooleanLit> for bool {
    fn from(value: BooleanLit) -> Self {
        match value {
            BooleanLit::True => true,
            BooleanLit::False => false,
        }
    }
}

pub async fn list(
    State(state): State<HttpState>,
    Query(params): Query<FlattenPageQueryWithFilter<ObjectFilterQuery>>,
) -> Response {
    let (page, filter) = params.into_parts();
    let filter: ObjectFilter = ObjectFilter {
        data_type: filter.data_type,
        id: filter.id,
        revision: filter.revision,
        latest_only: filter.latest_only.map(|v| v.into()),
        created_before: filter.created_before,
        created_after: filter.created_after,
    };
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
