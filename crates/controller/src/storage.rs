use std::sync::Arc;

use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use switchboard_custom_config::SerdeValue;
use switchboard_model::{Cursor, FlattenPageQueryWithFilter, PageQuery, PagedList, ServiceConfig};

use crate::ControllerContext;
pub mod surrealdb_local;

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StorageProvider {
    Local { db_file: std::path::PathBuf },
}

impl Default for StorageProvider {
    fn default() -> Self {
        StorageProvider::Local {
            db_file: crate::dir::config_local_db_path(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct StorageObjectDescriptor {
    pub id: String,
    pub revision: String,
}

impl std::fmt::Display for StorageObjectDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.id, self.revision)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StorageMeta {
    pub data_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StorageObject {
    pub descriptor: StorageObjectDescriptor,
    pub meta: StorageMeta,
    pub data: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StorageObjectWithoutData {
    pub descriptor: StorageObjectDescriptor,
    pub meta: StorageMeta,
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Object not found: {descriptor}")]
    ObjectNotFound { descriptor: StorageObjectDescriptor },
    #[error("Storage {provider} error: {source}")]
    StorageError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        provider: &'static str,
    },
    #[error("Encode error: {0}")]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error("Decode error: {0}")]
    DecodeError(#[from] bincode::error::DecodeError),
    #[error("Serialization error: {0}")]
    SerializationError(#[source] switchboard_custom_config::SerdeValueError),
    #[error("Deserialization error: {0}")]
    DeserializationError(#[source] switchboard_custom_config::SerdeValueError),
    #[error("Digest mismatch: expected {expected}, found {found}")]
    DigestMismatch { expected: String, found: String },
}
pub fn encode_object(object: SerdeValue) -> Result<(String, Vec<u8>), StorageError> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let bytes = bincode::encode_to_vec(object, bincode::config::standard())?;
    hasher.update(bytes.len().to_be_bytes());
    hasher.update([0]);
    hasher.update(&bytes);
    let revision = hasher.finalize();
    let revision_hex = hex::encode(revision);
    Ok((revision_hex, bytes))
}
pub fn decode_object(bytes: &[u8], digest: &str) -> Result<SerdeValue, StorageError> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes.len().to_be_bytes());
    hasher.update([0]);
    hasher.update(bytes);
    let recalculated_digest = hasher.finalize();
    let recalculated_hex = hex::encode(recalculated_digest);
    // check digest matches
    if digest != recalculated_hex {
        return Err(StorageError::DigestMismatch {
            expected: digest.to_string(),
            found: recalculated_hex,
        });
    }
    let (object, _): (SerdeValue, _) =
        bincode::decode_from_slice(bytes, bincode::config::standard())?;
    Ok(object)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct ObjectFilter {
    pub data_type: Option<String>,
    pub id: Option<String>,
    pub revision: Option<String>,
    pub latest_only: Option<bool>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

pub struct StorageObjectValueStyle {
    pub descriptor: StorageObjectDescriptor,
    pub meta: StorageMeta,
    pub data: SerdeValue,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ListObjectQuery {
    pub filter: ObjectFilter,
    pub page: PageQuery,
}

impl ListObjectQuery {
    pub fn into_binds(self) -> FlattenPageQueryWithFilter<ObjectFilter> {
        FlattenPageQueryWithFilter {
            next: self.page.cursor.next,
            limit: self.page.limit,
            filter: self.filter,
        }
    }
}

pub trait Storage: Send + Sync + 'static {
    fn save_object(
        &self,
        name: &str,
        data_type: &str,
        object: SerdeValue,
    ) -> impl Future<Output = Result<StorageObjectDescriptor, StorageError>> + Send;

    fn get_object(
        &self,
        descriptor: &StorageObjectDescriptor,
    ) -> impl Future<Output = Result<Option<StorageObjectValueStyle>, StorageError>> + Send;

    fn list_objects(
        &self,
        query: ListObjectQuery,
    ) -> impl Future<Output = Result<PagedList<StorageObjectWithoutData>, StorageError>> + Send;

    fn delete_object(
        &self,
        descriptor: &StorageObjectDescriptor,
    ) -> impl Future<Output = Result<Option<StorageObjectDescriptor>, StorageError>> + Send;

    fn batch_delete_objects(
        &self,
        descriptors: Vec<StorageObjectDescriptor>,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    fn delete_all_objects_by_id(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
}

pub trait DynamicStorage: Send + Sync + 'static {
    fn save_object<'a>(
        &'a self,
        id: &'a str,
        data_type: &'a str,
        object: SerdeValue,
    ) -> BoxFuture<'a, Result<StorageObjectDescriptor, StorageError>>;
    fn list_objects(
        &self,
        query: ListObjectQuery,
    ) -> BoxFuture<'_, Result<PagedList<StorageObjectWithoutData>, StorageError>>;
    fn get_object<'a>(
        &'a self,
        descriptor: &'a StorageObjectDescriptor,
    ) -> BoxFuture<'a, Result<Option<StorageObjectValueStyle>, StorageError>>;
    fn delete_object<'a>(
        &'a self,
        descriptor: &'a StorageObjectDescriptor,
    ) -> BoxFuture<'a, Result<Option<StorageObjectDescriptor>, StorageError>>;
    fn batch_delete_objects(
        &self,
        descriptors: Vec<StorageObjectDescriptor>,
    ) -> BoxFuture<'_, Result<(), StorageError>>;

    fn delete_all_objects_by_id<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), StorageError>>;
}

impl<S: Storage> DynamicStorage for S {
    fn save_object<'a>(
        &'a self,
        id: &'a str,
        data_type: &'a str,
        object: SerdeValue,
    ) -> BoxFuture<'a, Result<StorageObjectDescriptor, StorageError>> {
        Box::pin(self.save_object(id, data_type, object))
    }

    fn list_objects(
        &self,
        query: ListObjectQuery,
    ) -> BoxFuture<'_, Result<PagedList<StorageObjectWithoutData>, StorageError>> {
        Box::pin(self.list_objects(query))
    }

    fn get_object<'a>(
        &'a self,
        descriptor: &'a StorageObjectDescriptor,
    ) -> BoxFuture<'a, Result<Option<StorageObjectValueStyle>, StorageError>> {
        Box::pin(self.get_object(descriptor))
    }

    fn delete_object<'a>(
        &'a self,
        descriptor: &'a StorageObjectDescriptor,
    ) -> BoxFuture<'a, Result<Option<StorageObjectDescriptor>, StorageError>> {
        Box::pin(self.delete_object(descriptor))
    }

    fn batch_delete_objects(
        &self,
        descriptors: Vec<StorageObjectDescriptor>,
    ) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(self.batch_delete_objects(descriptors))
    }

    fn delete_all_objects_by_id<'a>(
        &'a self,
        id: &'a str,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(self.delete_all_objects_by_id(id))
    }
}

pub type SharedStorage = Arc<dyn DynamicStorage>;

pub trait KnownStorageObject: Serialize + DeserializeOwned + Send + Sync + 'static {
    fn data_type() -> &'static str;
}

mod impl_static_storage_object {
    use super::KnownStorageObject;
    macro_rules! derive_local_type {
        ($type: ty) => {
            impl KnownStorageObject for $type {
                fn data_type() -> &'static str {
                    stringify!($type)
                }
            }
        };
    }
    type ServiceConfig = switchboard_model::ServiceConfig;
    derive_local_type!(ServiceConfig);
}

pub(crate) async fn create_storage(
    provider: &StorageProvider,
) -> Result<SharedStorage, StorageError> {
    match provider {
        StorageProvider::Local { db_file } => {
            let storage = surrealdb_local::SurrealRocksDbStorage::new(db_file.as_ref()).await?;
            Ok(Arc::new(storage))
        }
    }
}

impl ControllerContext {
    pub async fn create_storage(&self) -> Result<SharedStorage, StorageError> {
        create_storage(&self.controller_config.storage).await
    }
    pub fn storage(&self) -> &SharedStorage {
        &self.storage
    }
    pub async fn save_known_object<T: KnownStorageObject>(
        &self,
        id: &str,
        object: T,
    ) -> Result<StorageObjectDescriptor, StorageError> {
        let serde_value =
            SerdeValue::serialize_from(&object).map_err(StorageError::SerializationError)?;
        self.storage
            .save_object(id, T::data_type(), serde_value)
            .await
    }
}

pub struct JsonInterpreter;

#[derive(Debug, thiserror::Error)]
pub enum JsonInterpreterError {
    #[error("Unsupported data type for JSON interpretation: {0}")]
    UnsupportedDataType(String),
    #[error("Serde Value error: {0}")]
    SerdeValueError(#[from] switchboard_custom_config::SerdeValueError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl JsonInterpreter {
    pub fn encode(obj: StorageObjectValueStyle) -> Result<serde_json::Value, JsonInterpreterError> {
        let data_type = obj.meta.data_type.as_str();
        if data_type == ServiceConfig::data_type() {
            let config: ServiceConfig = obj
                .data
                .deserialize_into()
                .map_err(JsonInterpreterError::SerdeValueError)?;
            Ok(serde_json::to_value(&config).map_err(JsonInterpreterError::Json)?)
        } else {
            Err(JsonInterpreterError::UnsupportedDataType(
                data_type.to_string(),
            ))
        }
    }
    pub fn decode(
        data: serde_json::Value,
        data_type: &str,
    ) -> Result<SerdeValue, JsonInterpreterError> {
        if data_type == ServiceConfig::data_type() {
            let config: ServiceConfig =
                serde_json::from_value(data).map_err(JsonInterpreterError::Json)?;
            let serde_value = SerdeValue::serialize_from(&config)
                .map_err(JsonInterpreterError::SerdeValueError)?;
            Ok(serde_value)
        } else {
            Err(JsonInterpreterError::UnsupportedDataType(
                data_type.to_string(),
            ))
        }
    }
}
