use std::path::Path;

use chrono::Utc;
use surrealdb::{
    RecordId, RecordIdKey, Surreal,
    engine::local::{Db, RocksDb},
    opt::{CreateResource, IntoResource},
};
use switchboard_custom_config::SerdeValue;
use switchboard_model::{Cursor, Indexed, PageQuery};

use crate::storage::{
    ListObjectQuery, Storage, StorageError, StorageMeta, StorageObject, StorageObjectDescriptor,
    StorageObjectWithoutData, decode_object,
};
pub struct SurrealRocksDbStorage {
    pub client: Surreal<Db>,
}
const OBJECT_TABLE: &str = "storage_object";
const PROVIDER: &str = "SurrealDB";
type FlattenedListObjectQuery = switchboard_model::FlattenPageQueryWithFilter<super::ObjectFilter>;
impl StorageObjectDescriptor {
    fn surreal_db_record_id(&self) -> RecordId {
        RecordId::from_table_key(OBJECT_TABLE, self.id())
    }
    fn id(&self) -> String {
        format!("{}:{}", self.id, self.revision)
    }
    fn from_surreal_db_record_id(record_id: &RecordIdKey) -> Result<Self, StorageError> {
        record_id
            .to_string()
            .split_once(':')
            .map(|(name, revision)| StorageObjectDescriptor {
                id: name.to_string(),
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
        let client = Surreal::new::<RocksDb>(db_dir)
            .await
            .map_err(storage_error)?;
        let this = Self { client };
        // this.ensure_initialized().await?;
        Ok(this)
    }
    // pub async fn ensure_initialized(&self) -> Result<(), StorageError> {
    //     self.client
    //         .query(format!("CREATE TABLE IF NOT EXISTS {};", OBJECT_TABLE))
    //         .await
    //         .map_err(storage_error)?;
    //     Ok(())
    // }
}

impl<D> IntoResource<D> for &StorageObjectDescriptor {
    fn into_resource(self) -> surrealdb::Result<surrealdb::opt::Resource> {
        let resource = surrealdb::opt::Resource::RecordId(self.surreal_db_record_id());
        Ok(resource)
    }
}

impl CreateResource<Option<RecordId>> for &StorageObjectDescriptor {
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
    async fn get_object(
        &self,
        descriptor: &StorageObjectDescriptor,
    ) -> Result<SerdeValue, StorageError> {
        let result: Option<StorageObject> = self
            .client
            .select::<Option<StorageObject>>(descriptor)
            .await
            .map_err(storage_error)?;
        let model = result.ok_or(StorageError::ObjectNotFound {
            descriptor: descriptor.clone(),
        })?;
        decode_object(&model.data, &model.descriptor.revision)
    }
    async fn save_object(
        &self,
        name: &str,
        data_type: &str,
        object: SerdeValue,
    ) -> Result<StorageObjectDescriptor, StorageError> {
        let (revision, data) = super::encode_object(object)?;
        let now = Utc::now();
        let model = StorageObject {
            descriptor: StorageObjectDescriptor {
                id: name.to_string(),
                revision,
            },
            meta: StorageMeta {
                created_at: now,
                data_type: data_type.to_string(),
            },
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
                    StorageObjectDescriptor::from_surreal_db_record_id(record_id.key())?;
                Ok(descriptor)
            }
            None => Err(StorageError::StorageError {
                source: "Failed to retrieve inserted config ID".into(),
                provider: PROVIDER,
            }),
        }
    }
    async fn delete_object(
        &self,
        descriptor: &StorageObjectDescriptor,
    ) -> Result<(), StorageError> {
        let deleted = self
            .client
            .delete::<Option<StorageObjectWithoutData>>(descriptor)
            .await
            .map_err(storage_error)?;
        if deleted.is_none() {
            return Err(StorageError::ObjectNotFound {
                descriptor: descriptor.clone(),
            });
        }
        Ok(())
    }
    async fn list_objects(
        &self,
        list_object_query: ListObjectQuery,
    ) -> Result<switchboard_model::PagedResult<StorageObjectWithoutData>, StorageError> {
        let ListObjectQuery { filter, page } = &list_object_query;
        let mut where_clauses = vec![];
        if page.cursor.next.is_some() {
            where_clauses.push("id > $cursor");
        };
        if filter.data_type.is_some() {
            where_clauses.push("meta.data_type = $data_type");
        }
        if filter.id.is_some() {
            where_clauses.push("descriptor.id = $id");
        }
        if filter.revision.is_some() {
            where_clauses.push("descriptor.revision = $revision");
        }
        if filter.created_before.is_some() {
            where_clauses.push("meta.created_at < $created_before");
        }
        if filter.created_after.is_some() {
            where_clauses.push("meta.created_at > $created_after");
        }
        if let Some(true) = filter.latest_only {
            where_clauses.push(
                "descriptor.revision = (SELECT MAX(descriptor.revision) FROM type::table($table) WHERE descriptor.id = descriptor.id)",
            );
        }
        let where_clause = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };
        let sql = format!(
            "SELECT * FROM type::table($table) {} ORDER BY id LIMIT $limit",
            where_clause
        );

        let query = self
            .client
            .query(sql)
            .bind(list_object_query.into_binds())
            .bind(("$table", OBJECT_TABLE));
        let items = query
            .await
            .map_err(storage_error)?
            .take::<Vec<StorageObjectWithoutData>>(0)
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

    async fn delete_all_objects_by_id(&self, names: &str) -> Result<(), StorageError> {
        let sql = "DELETE FROM type::table($table) WHERE descriptor.name = $name";
        self.client
            .query(sql)
            .bind(("table", OBJECT_TABLE))
            .bind(("name", names.to_string()))
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn batch_delete_objects(
        &self,
        descriptors: Vec<StorageObjectDescriptor>,
    ) -> Result<(), StorageError> {
        let ids: Vec<RecordId> = descriptors
            .iter()
            .map(|desc| desc.surreal_db_record_id())
            .collect();
        let sql = "DELETE FROM type::table($table) WHERE id IN $ids";
        self.client
            .query(sql)
            .bind(("table", OBJECT_TABLE))
            .bind(("ids", ids))
            .await
            .map_err(storage_error)?;
        Ok(())
    }
}
