use std::path::Path;

use chrono::Utc;
use surrealdb::{
    RecordId, RecordIdKey, Surreal,
    engine::local::{Db, RocksDb},
    opt::{CreateResource, IntoResource},
};
use switchboard_custom_config::SerdeValue;
use switchboard_model::{Cursor, Indexed, PageQuery};
use surrealdb::sql::Thing;
use crate::storage::{
    ListObjectQuery, Storage, StorageError, StorageMeta, StorageObject, StorageObjectDescriptor,
    StorageObjectWithoutData, decode_object,
};
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
        let sql = format!(
            "DEFINE TABLE {OBJECT_LATEST_INDEX_TABLE} SCHEMAFULL;
             DEFINE FIELD id ON {OBJECT_LATEST_INDEX_TABLE} TYPE string;
             DEFINE FIELD revision ON {OBJECT_LATEST_INDEX_TABLE} TYPE string;
             DEFINE FIELD record_id ON {OBJECT_LATEST_INDEX_TABLE} TYPE record;
             DEFINE INDEX unique_id ON {OBJECT_LATEST_INDEX_TABLE} COLUMNS id UNIQUE;",
        );
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

        let sql = format!(
            "
            LET $record = upsert only type::table($table) CONTENT $content;\
            UPDATE $latest_table: \
            CONTENT {{ id: $id, revision: $revision, record_id: result::id }} \
            WHERE id = $id;
            "
        );
        let insert_result = self
            .client
            .insert::<Option<Thing>>(&model.descriptor)
            .content(model)
            .await;
        let thing = match insert_result {
            Ok(thing) => thing,
            Err(surrealdb::Error::Db(surrealdb::error::Db::RecordExists { thing })) => {
                Some(thing)
            }
            Err(e) => {
                return Err(storage_error(e));   
            }
        };
        match thing {
            Some(thing) => {
                let descriptor =
                    StorageObjectDescriptor::from_id(&thing.id.to_raw())?;
                self.update_latest_index(&descriptor.id, &descriptor.revision, thing)
                    .await?;
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

        let base_table = if filter.latest_only == Some(true) {
            format!(
                "(SELECT obj.* FROM type::table($table) AS obj 
                 INNER JOIN type::table($latest_table) AS latest 
                 ON obj.id = latest.record_id)"
            )
        } else {
            "type::table($table)".to_string()
        };
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
        let sql = format!("SELECT * FROM {base_table} {where_clause} ORDER BY id LIMIT $limit");

        let query = self
            .client
            .query(sql)
            .bind(list_object_query.into_binds())
            .bind(("table", OBJECT_TABLE))
            .bind(("latest_table", OBJECT_LATEST_INDEX_TABLE));
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
