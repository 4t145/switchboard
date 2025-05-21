use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use surrealdb::Surreal;

impl switchboard_model::ConfigService for FsConfigService {
    type Error = FsConfigServiceError;

    fn get_many_items(
        &self,
        query: ItemQuery,
        cursor: CursorQuery,
    ) -> impl Future<Output = Result<PagedResult<Item>, Self::Error>> + Send + '_ {
        async { todo!() }
    }

    fn get_item_by_id(
        &self,
        id: String,
    ) -> impl Future<Output = Result<Option<Item>, Self::Error>> + Send + '_ {
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
        items: Vec<Item>,
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
        items: std::collections::HashMap<String, Item>,
    ) -> impl Future<Output = Result<Vec<Result<(), Self::Error>>, Self::Error>> + Send + '_ {
        async { todo!() }
    }

    async fn get_named_service_config(
        &self,
        name: String,
    ) -> Result<Option<impl tokio::io::AsyncRead + Send>, Self::Error> {
        let this = self.inner.read().await;
        let config = this
            .memory_config
            .services
            .get(&name)
            .ok_or(FsConfigServiceError::ServiceNotFound)?;
        match config {
            Some(config) => {
                let read = config.read().await?;
                Ok(Some(read))
            }
            None => Ok(None),
        }
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
