use std::sync::Arc;

use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use switchboard_model::{Config, CursorQuery, PagedResult};

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
pub struct StorageConfigDescriptor {
    pub name: String,
    pub revision: String,
}

impl std::fmt::Display for StorageConfigDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.revision)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StorageMeta {
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StorageConfig {
    pub descriptor: StorageConfigDescriptor,
    pub meta: StorageMeta,
    pub data: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StorageConfigWithoutData {
    pub descriptor: StorageConfigDescriptor,
    pub meta: StorageMeta,
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Configuration not found: {descriptor}")]
    ConfigNotFound { descriptor: StorageConfigDescriptor },
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
    #[error("Digest mismatch: expected {expected}, found {found}")]
    DigestMismatch { expected: String, found: String },
}

pub fn encode_config(config: &Config) -> Result<(String, Vec<u8>), StorageError> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let bytes = bincode::encode_to_vec(config, bincode::config::standard())?;
    hasher.update(bytes.len().to_be_bytes());
    hasher.update([0]);
    hasher.update(&bytes);
    let revision = hasher.finalize();
    let revision_hex = hex::encode(revision);
    Ok((revision_hex, bytes))
}
pub fn decode_config(bytes: &[u8], digest: &str) -> Result<Config, StorageError> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes.len().to_be_bytes());
    hasher.update([0]);
    hasher.update(&bytes);
    let recalculated_digest = hasher.finalize();
    let recalculated_hex = hex::encode(recalculated_digest);
    // check digest matches
    if digest != &recalculated_hex {
        return Err(StorageError::DigestMismatch {
            expected: digest.to_string(),
            found: recalculated_hex,
        });
    }
    let (config, _): (Config, _) = bincode::decode_from_slice(bytes, bincode::config::standard())?;
    Ok(config)
}
pub trait Storage: Send + Sync + 'static {
    fn save_config(
        &self,
        name: &str,
        config: Config,
    ) -> impl Future<Output = Result<StorageConfigDescriptor, StorageError>> + Send;

    fn list_configs(
        &self,
        query: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<StorageConfigWithoutData>, StorageError>> + Send;

    fn list_latest_configs(
        &self,
        query: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<StorageConfigWithoutData>, StorageError>> + Send;

    fn list_configs_by_name(
        &self,
        name: &str,
        query: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<StorageConfigWithoutData>, StorageError>> + Send;

    fn get_config(
        &self,
        descriptor: &StorageConfigDescriptor,
    ) -> impl Future<Output = Result<Config, StorageError>> + Send;

    fn delete_config(
        &self,
        descriptor: &StorageConfigDescriptor,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    fn batch_delete_configs(
        &self,
        descriptors: Vec<StorageConfigDescriptor>,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    fn delete_all_config_by_name(
        &self,
        names: &str,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
}

pub trait DynamicStorage: Send + Sync + 'static {
    fn save_config<'a>(
        &'a self,
        name: &'a str,
        config: Config,
    ) -> BoxFuture<'a, Result<StorageConfigDescriptor, StorageError>>;
    fn list_configs(
        &self,
        query: CursorQuery,
    ) -> BoxFuture<'_, Result<PagedResult<StorageConfigWithoutData>, StorageError>>;

    fn list_latest_configs(
        &self,
        query: CursorQuery,
    ) -> BoxFuture<'_, Result<PagedResult<StorageConfigWithoutData>, StorageError>>;

    fn list_configs_by_name<'a>(
        &'a self,
        name: &'a str,
        query: CursorQuery,
    ) -> BoxFuture<'a, Result<PagedResult<StorageConfigWithoutData>, StorageError>>;
    fn get_config<'a>(
        &'a self,
        descriptor: &'a StorageConfigDescriptor,
    ) -> BoxFuture<'a, Result<Config, StorageError>>;
    fn delete_config<'a>(
        &'a self,
        descriptor: &'a StorageConfigDescriptor,
    ) -> BoxFuture<'a, Result<(), StorageError>>;
    fn batch_delete_configs(
        &self,
        descriptors: Vec<StorageConfigDescriptor>,
    ) -> BoxFuture<'_, Result<(), StorageError>>;

    fn delete_all_config_by_name<'a>(
        &'a self,
        names: &'a str,
    ) -> BoxFuture<'a, Result<(), StorageError>>;
}

impl<S: Storage> DynamicStorage for S {
    fn save_config<'a>(
        &'a self,
        name: &'a str,
        config: Config,
    ) -> BoxFuture<'a, Result<StorageConfigDescriptor, StorageError>> {
        Box::pin(self.save_config(name, config))
    }

    fn list_configs(
        &self,
        query: CursorQuery,
    ) -> BoxFuture<'_, Result<PagedResult<StorageConfigWithoutData>, StorageError>> {
        Box::pin(self.list_configs(query))
    }

    fn list_latest_configs(
        &self,
        query: CursorQuery,
    ) -> BoxFuture<'_, Result<PagedResult<StorageConfigWithoutData>, StorageError>> {
        Box::pin(self.list_latest_configs(query))
    }

    fn list_configs_by_name<'a>(
        &'a self,
        name: &'a str,
        query: CursorQuery,
    ) -> BoxFuture<'a, Result<PagedResult<StorageConfigWithoutData>, StorageError>> {
        Box::pin(self.list_configs_by_name(name, query))
    }

    fn get_config<'a>(
        &'a self,
        descriptor: &'a StorageConfigDescriptor,
    ) -> BoxFuture<'a, Result<Config, StorageError>> {
        Box::pin(self.get_config(descriptor))
    }

    fn delete_config<'a>(
        &'a self,
        descriptor: &'a StorageConfigDescriptor,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(self.delete_config(descriptor))
    }

    fn batch_delete_configs(
        &self,
        descriptors: Vec<StorageConfigDescriptor>,
    ) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(self.batch_delete_configs(descriptors))
    }

    fn delete_all_config_by_name<'a>(
        &'a self,
        names: &'a str,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(self.delete_all_config_by_name(names))
    }
}

pub type SharedStorage = Arc<dyn DynamicStorage>;

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
    pub async fn get_storage(&self) -> Result<SharedStorage, StorageError> {
        create_storage(&self.controller_config.storage).await
    }
}