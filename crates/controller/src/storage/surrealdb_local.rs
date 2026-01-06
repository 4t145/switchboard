use std::path::Path;

use crate::storage::{
    ListObjectQuery, Storage, StorageError, StorageMeta, StorageObject, StorageObjectDescriptor,
    StorageObjectWithoutData, decode_object,
};
use chrono::Utc;
use surrealdb::sql::Thing;
use surrealdb::{
    RecordId, RecordIdKey, Surreal,
    engine::local::{Db, RocksDb},
    opt::{CreateResource, IntoResource},
};
use switchboard_custom_config::SerdeValue;
use switchboard_model::{Cursor, Indexed, PageQuery};
pub struct SurrealRocksDbStorage {
    pub client: Surreal<Db>,
}
const OBJECT_TABLE: &str = "storage_object";
const OBJECT_LATEST_INDEX_TABLE: &str = "storage_object_latest";

// 新增：最新版本索引结构
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LatestRevisionIndex {
    id: String,          // 对象 ID
    revision: String,    // 最新版本号
    record_id: RecordId, // 指向实际记录的 ID
}

const PROVIDER: &str = "SurrealDB";
type FlattenedListObjectQuery = switchboard_model::FlattenPageQueryWithFilter<super::ObjectFilter>;
impl StorageObjectDescriptor {
    fn surreal_db_record_id(&self) -> RecordId {
        RecordId::from_table_key(OBJECT_TABLE, self.id())
    }
    fn id(&self) -> String {
        format!("{}:{}", self.id, self.revision)
    }
    fn from_id(record_id: &str) -> Result<Self, StorageError> {
        record_id
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

    // 更新最新版本索引
    async fn update_latest_index(
        &self,
        object_id: &str,
        revision: &str,
        record_id: RecordId,
    ) -> Result<(), StorageError> {
        let sql = format!(
            "UPDATE type::table($table) 
             CONTENT {{ id: $object_id, revision: $revision, record_id: $record_id }}
             WHERE id = $object_id"
        );

        self.client
            .query(sql)
            .bind(("table", OBJECT_LATEST_INDEX_TABLE))
            .bind(("object_id", object_id.to_string()))
            .bind(("revision", revision.to_string()))
            .bind(("record_id", record_id))
            .await
            .map_err(storage_error)?;

        Ok(())
    }

    // 删除最新版本索引
    async fn delete_latest_index(&self, object_id: &str) -> Result<(), StorageError> {
        let sql = "DELETE FROM type::table($table) WHERE id = $object_id";
        self.client
            .query(sql)
            .bind(("table", OBJECT_LATEST_INDEX_TABLE))
            .bind(("object_id", object_id.to_string()))
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // 重新计算并更新某个对象的最新版本索引
    async fn recalculate_latest_index(&self, object_id: &str) -> Result<(), StorageError> {
        // 查询该对象的最新版本
        let sql = "
            SELECT * FROM type::table($table) 
            WHERE descriptor.id = $object_id 
            ORDER BY descriptor.revision DESC 
            LIMIT 1
        ";

        let mut result = self
            .client
            .query(sql)
            .bind(("table", OBJECT_TABLE))
            .bind(("object_id", object_id.to_string()))
            .await
            .map_err(storage_error)?;

        let latest: Option<StorageObjectWithoutData> = result.take(0).map_err(storage_error)?;

        if let Some(latest_obj) = latest {
            // 找到了新的最新版本，更新索引
            let record_id = latest_obj.descriptor.surreal_db_record_id();
            self.update_latest_index(object_id, &latest_obj.descriptor.revision, record_id)
                .await?;
        } else {
            // 没有找到任何版本，删除索引
            self.delete_latest_index(object_id).await?;
        }

        Ok(())
    }
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
    ) -> Result<SerdeValue, StorageError> {
        let result: Option<StorageObjectDb> = self
            .client
            .select::<Option<StorageObjectDb>>(descriptor)
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
                    created_before: filter
                        .created_before
                        .map(|dt| surrealdb::sql::Datetime::from(dt)),
                    created_after: filter
                        .created_after
                        .map(|dt| surrealdb::sql::Datetime::from(dt)),
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
        let sql = "
        DELETE FROM type::table($table) WHERE descriptor.id = $id;
        DELETE FROM type::table($latest_table) WHERE id = $id;
        ";
        self.client
            .query(sql)
            .bind(("table", OBJECT_TABLE))
            .bind(("latest_table", OBJECT_LATEST_INDEX_TABLE))
            .bind(("id", id.to_string()))
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn batch_delete_objects(
        &self,
        descriptors: Vec<StorageObjectDescriptor>,
    ) -> Result<(), StorageError> {
        use std::collections::HashSet;

        // 收集所有受影响的对象 ID
        let mut affected_object_ids = HashSet::new();
        for desc in &descriptors {
            affected_object_ids.insert(desc.id.clone());
        }

        // 执行删除操作
        let ids: Vec<RecordId> = descriptors
            .iter()
            .map(|desc| desc.surreal_db_record_id())
            .collect();
        let sql = "
        DELETE FROM type::table($table) WHERE id IN $ids;
        FOR $affected_object_id in $affected_object_ids {
            LET latest_index = (SELECT * FROM type::table($latest_table) WHERE id = $affected_object_id);
            IF (latest_index != NONE) {
                LET latest_record_id = latest_index[0].record_id;
                IF (latest_record_id != NONE) {
                    DELETE latest_record_id;
                }
            }
        }
        ";
        self.client
            .query(sql)
            .bind(("table", OBJECT_TABLE))
            .bind(("ids", ids))
            .await
            .map_err(storage_error)?;

        // // 对每个受影响的对象 ID 重新计算最新版本
        // for object_id in affected_object_ids {
        //     self.recalculate_latest_index(&object_id).await?;
        // }

        Ok(())
    }
}
