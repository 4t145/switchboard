use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{
    RecordId, Surreal, engine::local::{Db, RocksDb}, opt::IntoResource
};
use switchboard_custom_config::formats::encode_bytes;

use crate::storage::{
    Storage, StorageConfig, StorageConfigDescriptor, StorageConfigWithoutData, StorageError,
    StorageMeta,
};

pub struct SurrealRocksDbStorage {
    pub client: Surreal<Db>,
}

impl SurrealRocksDbStorage {
    pub async fn new() -> Result<Self, surrealdb::Error> {
        let client = Surreal::new::<RocksDb>("path/to/database-folder").await?;
        Ok(Self { client })
    }
}

impl IntoResource<StorageConfig> for &StorageConfigDescriptor {
    fn into_resource(self) -> surrealdb::Result<surrealdb::opt::Resource> {
        let resource = surrealdb::opt::Resource::RecordId(RecordId::from_table_key("config", format!(
            "{}:{}",
            self.name, self.revision
        )));
        Ok(resource)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfigModel {
    pub id: RecordId,
    pub name: String,
    pub revision: String,
    pub created_at: surrealdb::sql::Datetime,
    pub data: surrealdb::sql::Bytes,

}

impl Into<StorageConfig> for StorageConfigModel {
    fn into(self) -> StorageConfig {
        StorageConfig {
            descriptor: StorageConfigDescriptor {
                name: self.name,
                revision: self.revision,
            },
            meta: StorageMeta {
                created_at: self.created_at.0,
            },
            data: self.data.into_inner(),
        }
    }
}

impl From<StorageConfig> for StorageConfigModel {
    fn from(config: StorageConfig) -> Self {
        Self {
            id: RecordId::from_table_key(
                "config",
                format!("{}:{}", config.descriptor.name, config.descriptor.revision),
            ),
            name: config.descriptor.name,
            revision: config.descriptor.revision,
            created_at: surrealdb::sql::Datetime::from(config.meta.created_at),
            data: surrealdb::sql::Bytes::from(config.data),
        }
    }
}

impl Storage for SurrealRocksDbStorage {
    async fn get_config(
            &self,
            descriptor: &StorageConfigDescriptor,
        ) -> Result<StorageConfig, StorageError> {
        let result: Option<StorageConfigModel> = self.client.select(descriptor).await?;
        let model = result.ok_or(StorageError::ConfigNotFound {
            descriptor: descriptor.clone(),
        })?;
        Ok(model.into())
    }
    async fn save_config(
            &self,
            name: &str,
            config: switchboard_model::Config,
        ) -> Result<(), StorageError> {
        let (revision, data) = super::encode_config(&config)?;
        let now = Utc::now();
        let model = StorageConfigModel {
            id: RecordId::from_table_key("config", format!("{}:{}", name, revision)),
            name: name.to_string(),
            revision: revision.clone(),
            created_at: surrealdb::sql::Datetime::from(now),
            data: surrealdb::sql::Bytes::from(data),
        };
        Ok(model.into())
    }
}
