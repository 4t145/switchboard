use std::path::Path;

use crate::storage::{
    ListObjectQuery, Storage, StorageError, StorageMeta, StorageObject, StorageObjectDescriptor,
    StorageObjectValueStyle, StorageObjectWithoutData, decode_object,
};
use chrono::Utc;
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};
use switchboard_model::{Cursor, Indexed, SerdeValue};
pub struct SurrealRocksDbStorage {
    pub client: Surreal<Db>,
}

const PROVIDER: &str = "SurrealDB";

impl SurrealRocksDbStorage {
    pub async fn new(db_dir: &Path) -> Result<Self, StorageError> {
        let client = Surreal::new::<RocksDb>(db_dir)
            .await
            .map_err(storage_error)?;
        let this = Self { client };
        this.ensure_initialized().await?;
        Ok(this)
    }
    pub async fn ensure_initialized(&self) -> Result<(), StorageError> {
        // 定义索引表
        let sql = include_str!("surrealdb_local/define.surrealql");
        self.client
            .use_ns("switchboard")
            .use_db("storage")
            .await
            .map_err(storage_error)?;
        self.client.query(sql).await.map_err(storage_error)?;
        Ok(())
    }
}

fn storage_error(e: surrealdb::Error) -> StorageError {
    StorageError::StorageError {
        source: Box::new(e),
        provider: "SurrealDB",
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StorageObjectDb {
    pub descriptor: StorageObjectDescriptor,
    pub meta: StorageMetaDb,
    pub data: surrealdb::sql::Bytes,
}

impl StorageObjectDb {
    pub fn from_storage_object(object: StorageObject) -> StorageObjectDb {
        StorageObjectDb {
            descriptor: object.descriptor,
            meta: StorageMetaDb {
                data_type: object.meta.data_type,
                created_at: object.meta.created_at.into(),
            },
            data: object.data.into(),
        }
    }
    pub fn into_storage_object(self) -> StorageObject {
        StorageObject {
            descriptor: self.descriptor,
            meta: StorageMeta {
                data_type: self.meta.data_type,
                created_at: self.meta.created_at.into(),
            },
            data: self.data.into(),
        }
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StorageMetaDb {
    pub data_type: String,
    pub created_at: surrealdb::sql::Datetime,
}

impl Storage for SurrealRocksDbStorage {
    async fn get_object(
        &self,
        descriptor: &StorageObjectDescriptor,
    ) -> Result<Option<StorageObjectValueStyle>, StorageError> {
        let result: Option<StorageObjectDb> = self
            .client
            .query("fn::storage_object::get($descriptor)")
            .bind(("descriptor", descriptor.clone()))
            .await
            .map_err(storage_error)?
            .take(0)
            .map_err(storage_error)?;
        let Some(result) = result else {
            return Ok(None);
        };
        let value = decode_object(&result.data, &result.descriptor.revision)?;
        Ok(Some(StorageObjectValueStyle {
            descriptor: result.descriptor,
            meta: StorageMeta {
                data_type: result.meta.data_type,
                created_at: result.meta.created_at.into(),
            },
            data: value,
        }))
    }
    async fn save_object(
        &self,
        name: &str,
        data_type: &str,
        object: SerdeValue,
    ) -> Result<StorageObjectDescriptor, StorageError> {
        let (revision, data) = super::encode_object(object)?;
        let now = Utc::now();
        let model = StorageObjectDb {
            descriptor: StorageObjectDescriptor {
                id: name.to_string(),
                revision,
            },
            meta: StorageMetaDb {
                created_at: now.into(),
                data_type: data_type.to_string(),
            },
            data: data.into(),
        };

        let insert_result = self
            .client
            .query("fn::storage_object::save($content)")
            .bind(("content", model))
            .await
            .map_err(storage_error)?
            .take(0)
            .map_err(storage_error)?;
        match insert_result {
            Some(descriptor) => Ok(descriptor),
            None => Err(StorageError::StorageError {
                source: "Failed to retrieve inserted config ID".into(),
                provider: PROVIDER,
            }),
        }
    }
    async fn delete_object(
        &self,
        descriptor: &StorageObjectDescriptor,
    ) -> Result<Option<StorageObjectDescriptor>, StorageError> {
        let deleted: Option<StorageObjectDescriptor> = self
            .client
            .query("fn::storage_object::delete($descriptor)")
            .bind(("descriptor", descriptor.clone()))
            .await
            .map_err(storage_error)?
            .take(0)
            .map_err(storage_error)?;
        if deleted.is_none() {
            return Err(StorageError::ObjectNotFound {
                descriptor: descriptor.clone(),
            });
        }
        Ok(deleted)
    }
    async fn list_objects(
        &self,
        list_object_query: ListObjectQuery,
    ) -> Result<switchboard_model::PagedList<StorageObjectWithoutData>, StorageError> {
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
        pub struct ObjectFilterDb {
            pub data_type: Option<String>,
            pub id: Option<String>,
            pub revision: Option<String>,
            pub latest_only: Option<bool>,
            pub created_before: Option<surrealdb::sql::Datetime>,
            pub created_after: Option<surrealdb::sql::Datetime>,
        }

        impl ObjectFilterDb {
            fn from_model(filter: super::ObjectFilter) -> Self {
                Self {
                    data_type: filter.data_type,
                    id: filter.id,
                    revision: filter.revision,
                    latest_only: filter.latest_only,
                    created_before: filter.created_before.map(surrealdb::sql::Datetime::from),
                    created_after: filter.created_after.map(surrealdb::sql::Datetime::from),
                }
            }
        }
        let ListObjectQuery { filter, page } = &list_object_query;
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
        struct ResponseItem {
            descriptor: StorageObjectDescriptor,
            meta: StorageMetaDb,
            id: Thing,
        }
        let query = self
            .client
            .query("fn::storage_object::list($filter, $limit, $cursor)")
            .bind(("filter", ObjectFilterDb::from_model(filter.clone())))
            .bind(("limit", page.limit))
            .bind(("cursor", page.cursor.next.clone()));
        let items = query
            .await
            .map_err(storage_error)?
            .take::<Vec<ResponseItem>>(0)
            .map_err(storage_error)?
            .into_iter()
            .map(|item| {
                Indexed::new(
                    item.id.id.to_raw(),
                    StorageObjectWithoutData {
                        descriptor: item.descriptor,
                        meta: StorageMeta {
                            data_type: item.meta.data_type,
                            created_at: item.meta.created_at.into(),
                        },
                    },
                )
            })
            .collect::<Vec<_>>();
        let next_cursor = items.last().map(|config| config.id.clone());
        let next_cursor = Cursor::new(next_cursor);
        Ok(switchboard_model::PagedList {
            items,
            next_cursor: Some(next_cursor),
        })
    }

    async fn delete_all_objects_by_id(&self, id: &str) -> Result<(), StorageError> {
        self.client
            .query("fn::storage_object::delete_by_id($object_id)")
            .bind(("object_id", id.to_string()))
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn batch_delete_objects(
        &self,
        descriptors: Vec<StorageObjectDescriptor>,
    ) -> Result<(), StorageError> {
        self.client
            .query("fn::storage_object::batch_delete($descriptors)")
            .bind(("descriptors", descriptors))
            .await
            .map_err(storage_error)?;
        Ok(())
    }
}
