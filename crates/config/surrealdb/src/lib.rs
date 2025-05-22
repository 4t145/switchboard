use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use surrealdb::Connect;
use surrealdb::Connection;
use surrealdb::RecordId;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::RocksDb;
use switchboard_model::*;
pub struct SurrealDbConfigService {
    db: Connect<Db, Surreal<Db>>,
}

#[derive(Debug, Serialize)]
struct ItemModel {
    #[serde(skip_serializing)]
    service_id: String,
    bind_ip: IpAddr,
    bind_port: u16,
}

#[derive(Debug, Serialize)]
struct ItemTagRelation {
    item_id: String,
    tag: String,
}

#[derive(Debug, Serialize)]
pub struct Service {
    #[serde(skip_serializing)]
    id: String,
    service_name: Option<String>,
    discription: Option<String>,
    config: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SurrealDbConfigServiceError {}
impl switchboard_model::ConfigService for SurrealDbConfigService {
    type Error = SurrealDbConfigServiceError;

    fn get_many_binds(
        &self,
        query: BindQuery,
        cursor: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<Bind>, Self::Error>> + Send + '_ {
        let x = Surreal::new::<RocksDb>("address");
        async { todo!() }
    }

    fn get_item_by_id(
        &self,
        id: String,
    ) -> impl Future<Output = Result<Option<Bind>, Self::Error>> + Send + '_ {
        async { todo!() }
    }

    fn has_named_service(
        &self,
        name: String,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send + '_ {
        async { todo!() }
    }

    fn add_items(
        &self,
        items: Vec<Bind>,
    ) -> impl Future<Output = Result<Vec<Result<String, Self::Error>>, Self::Error>> + Send + '_
    {
        async { todo!() }
    }

    fn delete_items(
        &self,
        ids: Vec<String>,
    ) -> impl Future<Output = Result<Vec<Result<(), Self::Error>>, Self::Error>> + Send + '_ {
        async { todo!() }
    }

    fn update_items(
        &self,
        items: std::collections::HashMap<String, Bind>,
    ) -> impl Future<Output = Result<Vec<Result<(), Self::Error>>, Self::Error>> + Send + '_ {
        async { todo!() }
    }

    async fn get_named_service(
            &self,
            name: String,
        ) -> Result<NamedService, Self::Error> {
        
    }

    fn set_named_service_config(
        &self,
        name: String,
        config: impl tokio::io::AsyncRead + Send + 'static,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        async { todo!() }
    }

    fn is_enabled(
        &self,
        id: String,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send + '_ {
        async { todo!() }
    }

    fn set_enabled(
        &self,
        items: std::collections::HashMap<String, bool>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        async { todo!() }
    }

    fn get_enabled(
        &self,
        enabled: bool,
        query: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<String>, Self::Error>> + Send + '_ {
        async { todo!() }
    }

    fn listen(
        &self,
        items: Vec<String>,
    ) -> impl Future<Output = Result<impl ConfigListener, Self::Error>> + Send + '_ {
        async { todo!() }
    }
}
