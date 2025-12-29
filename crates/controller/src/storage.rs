use chrono::{DateTime, Utc};
use switchboard_model::{Config, CursorQuery, PagedResult};
pub mod surrealdb_local;

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
    let revision = sha2::Sha256::digest(bytes);
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
pub trait Storage {
    fn save_config(
        &self,
        name: &str,
        config: Config,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    fn list_configs(
        &self,
        query: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<StorageConfigWithoutData>, StorageError>> + Send;

    fn list_latest_configs(
        &self,
    ) -> impl Future<Output = Result<Vec<StorageConfigWithoutData>, StorageError>> + Send;

    fn list_configs_by_name(
        &self,
        name: &str,
        query: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<StorageConfigWithoutData>, StorageError>> + Send;

    fn get_config(
        &self,
        descriptor: &StorageConfigDescriptor,
    ) -> impl Future<Output = Result<StorageConfig, StorageError>> + Send;

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
