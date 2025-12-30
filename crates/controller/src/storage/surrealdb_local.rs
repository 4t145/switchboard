use std::path::Path;

use chrono::Utc;
use surrealdb::{
    RecordId, RecordIdKey, Surreal,
    engine::local::{Db, RocksDb},
    opt::{CreateResource, IntoResource},
};
use switchboard_model::{Cursor, CursorQuery, Indexed};

use crate::storage::{
    Storage, StorageConfig, StorageConfigDescriptor, StorageConfigWithoutData, StorageError,
    StorageMeta, decode_config,
};
pub struct SurrealRocksDbStorage {
    pub client: Surreal<Db>,
}
const CONFIG_TABLE: &str = "config";
const PROVIDER: &str = "SurrealDB";

impl StorageConfigDescriptor {
    fn surreal_db_record_id(&self) -> RecordId {
        RecordId::from_table_key(CONFIG_TABLE, self.id())
    }
    fn id(&self) -> String {
        format!("{}:{}", self.name, self.revision)
    }
    fn from_surreal_db_record_id(record_id: &RecordIdKey) -> Result<Self, StorageError> {
        record_id
            .to_string()
            .split_once(':')
            .map(|(name, revision)| StorageConfigDescriptor {
                name: name.to_string(),
                revision: revision.to_string(),
            })
            .ok_or_else(|| StorageError::StorageError {
                source: "Invalid config RecordId format".into(),
                provider: PROVIDER,
            })
    }
}

impl SurrealRocksDbStorage {
    pub async fn new(db_dir: &Path) -> Result<Self, StorageError> {
        let client = Surreal::new::<RocksDb>(db_dir).await.map_err(storage_error)?;
        Ok(Self { client })
    }
}

impl<D> IntoResource<D> for &StorageConfigDescriptor {
    fn into_resource(self) -> surrealdb::Result<surrealdb::opt::Resource> {
        let resource = surrealdb::opt::Resource::RecordId(self.surreal_db_record_id());
        Ok(resource)
    }
}

impl CreateResource<Option<RecordId>> for &StorageConfigDescriptor {
    fn into_resource(self) -> surrealdb::Result<surrealdb::opt::Resource> {
        let resource = surrealdb::opt::Resource::RecordId(self.surreal_db_record_id());
        Ok(resource)
    }
}

fn storage_error(e: surrealdb::Error) -> StorageError {
    StorageError::StorageError {
        source: Box::new(e),
        provider: "SurrealDB",
    }
}

impl Storage for SurrealRocksDbStorage {
    async fn get_config(
        &self,
        descriptor: &StorageConfigDescriptor,
    ) -> Result<switchboard_model::Config, StorageError> {
        let result: Option<StorageConfig> = self
            .client
            .select::<Option<StorageConfig>>(descriptor)
            .await
            .map_err(storage_error)?;
        let model = result.ok_or(StorageError::ConfigNotFound {
            descriptor: descriptor.clone(),
        })?;
        decode_config(&model.data, &model.descriptor.revision)
    }
    async fn save_config(
        &self,
        name: &str,
        config: switchboard_model::Config,
    ) -> Result<StorageConfigDescriptor, StorageError> {
        let (revision, data) = super::encode_config(&config)?;
        let now = Utc::now();
        let model = StorageConfig {
            descriptor: StorageConfigDescriptor {
                name: name.to_string(),
                revision,
            },
            meta: StorageMeta { created_at: now },
            data,
        };
        let id = self
            .client
            .insert::<Option<RecordId>>(&model.descriptor)
            .content(model)
            .await
            .map_err(storage_error)?;
        match id {
            Some(record_id) => {
                let descriptor =
                    StorageConfigDescriptor::from_surreal_db_record_id(record_id.key())?;
                Ok(descriptor)
            }
            None => Err(StorageError::StorageError {
                source: "Failed to retrieve inserted config ID".into(),
                provider: PROVIDER,
            }),
        }
    }
    async fn delete_config(
        &self,
        descriptor: &StorageConfigDescriptor,
    ) -> Result<(), StorageError> {
        let deleted = self
            .client
            .delete::<Option<StorageConfig>>(descriptor)
            .await
            .map_err(storage_error)?;
        if deleted.is_none() {
            return Err(StorageError::ConfigNotFound {
                descriptor: descriptor.clone(),
            });
        }
        Ok(())
    }
    async fn list_configs(
        &self,
        cursor_query: switchboard_model::CursorQuery,
    ) -> Result<switchboard_model::PagedResult<StorageConfigWithoutData>, StorageError> {
        let sql = if !cursor_query.cursor.is_empty() {
            format!(
                "SELECT * OMIT data FROM type::table($table) \
                 WHERE id > type::thing($cursor) \
                 ORDER BY id ASC \
                 LIMIT $limit"
            )
        } else {
            format!(
                "SELECT * OMIT data FROM type::table($table) \
                 ORDER BY id ASC \
                 LIMIT $limit"
            )
        };

        let mut query = self
            .client
            .query(sql)
            .bind(("table", CONFIG_TABLE))
            .bind(("limit", cursor_query.limit));
        if let Some(cursor) = cursor_query.cursor.next {
            query = query.bind(("cursor", cursor));
        }
        let items = query
            .await
            .map_err(storage_error)?
            .take::<Vec<StorageConfigWithoutData>>(0)
            .map_err(storage_error)?
            .into_iter()
            .map(|item| Indexed::new(item.descriptor.id(), item))
            .collect::<Vec<_>>();
        let next_cursor = items.last().map(|config| config.id.clone());
        let next_cursor = Cursor::new(next_cursor);
        Ok(switchboard_model::PagedResult {
            items,
            next_cursor: Some(next_cursor),
        })
    }
    async fn list_configs_by_name(
        &self,
        name: &str,
        cursor_query: CursorQuery,
    ) -> Result<switchboard_model::PagedResult<StorageConfigWithoutData>, StorageError> {
        let sql = if !cursor_query.cursor.is_empty() {
            format!(
                "SELECT * OMIT data FROM type::table($table) \
                 WHERE descriptor.name = $name AND id > type::thing($cursor) \
                 ORDER BY id ASC \
                 LIMIT $limit"
            )
        } else {
            format!(
                "SELECT * OMIT data FROM type::table($table) \
                 WHERE descriptor.name = $name \
                 ORDER BY id ASC \
                 LIMIT $limit"
            )
        };
        let mut query = self
            .client
            .query(sql)
            .bind(("table", CONFIG_TABLE))
            .bind(("name", name.to_string()))
            .bind(("limit", cursor_query.limit));
        if let Some(cursor) = cursor_query.cursor.next {
            query = query.bind(("cursor", cursor));
        }
        let items = query
            .await
            .map_err(storage_error)?
            .take::<Vec<StorageConfigWithoutData>>(0)
            .map_err(storage_error)?
            .into_iter()
            .map(|item| Indexed::new(item.descriptor.id(), item))
            .collect::<Vec<_>>();
        let next_cursor = items.last().map(|config| config.id.clone());
        let next_cursor = Cursor::new(next_cursor);
        Ok(switchboard_model::PagedResult {
            items,
            next_cursor: Some(next_cursor),
        })
    }

    async fn list_latest_configs(
        &self,
        cursor_query: CursorQuery,
    ) -> Result<switchboard_model::PagedResult<StorageConfigWithoutData>, StorageError> {
        let sql = if !cursor_query.cursor.is_empty() {
            format!(
                "SELECT * OMIT data FROM type::table($table) \
                 WHERE id IN (SELECT max(id) FROM type::table($table) GROUP BY descriptor.name) \
                 AND id > type::thing($cursor) \
                 ORDER BY id ASC \
                 LIMIT $limit"
            )
        } else {
            format!(
                "SELECT * OMIT data FROM type::table($table) \
                 WHERE id IN (SELECT max(id) FROM type::table($table) GROUP BY descriptor.name) \
                 ORDER BY id ASC \
                 LIMIT $limit"
            )
        };
        let mut query = self
            .client
            .query(sql)
            .bind(("table", CONFIG_TABLE))
            .bind(("limit", cursor_query.limit));
        if let Some(cursor) = cursor_query.cursor.next {
            query = query.bind(("cursor", cursor));
        }
        let items = query
            .await
            .map_err(storage_error)?
            .take::<Vec<StorageConfigWithoutData>>(0)
            .map_err(storage_error)?
            .into_iter()
            .map(|item| Indexed::new(item.descriptor.id(), item))
            .collect::<Vec<_>>();
        let next_cursor = items.last().map(|config| config.id.clone());
        let next_cursor = Cursor::new(next_cursor);
        Ok(switchboard_model::PagedResult {
            items,
            next_cursor: Some(next_cursor),
        })
    }

    async fn delete_all_config_by_name(&self, names: &str) -> Result<(), StorageError> {
        let sql = "DELETE FROM type::table($table) WHERE descriptor.name = $name";
        self.client
            .query(sql)
            .bind(("table", CONFIG_TABLE))
            .bind(("name", names.to_string()))
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn batch_delete_configs(
        &self,
        descriptors: Vec<StorageConfigDescriptor>,
    ) -> Result<(), StorageError> {
        let ids: Vec<RecordId> = descriptors
            .iter()
            .map(|desc| desc.surreal_db_record_id())
            .collect();
        let sql = "DELETE FROM type::table($table) WHERE id IN $ids";
        self.client
            .query(sql)
            .bind(("table", CONFIG_TABLE))
            .bind(("ids", ids))
            .await
            .map_err(storage_error)?;
        Ok(())
    }
}
