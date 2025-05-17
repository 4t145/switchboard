use std::sync::Arc;
use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use switchboard_model::ServiceDescriptor;

use switchboard_model::*;
use tokio::fs::File;
use tokio::io::AsyncRead;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub enabled: Vec<ServiceDescriptor>,
    pub services: BTreeMap<String, Option<ServiceConfig>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ServiceConfig {
    Literal(String),
    File { path: PathBuf },
}

impl ServiceConfig {
    pub async fn read(&self) -> std::io::Result<ServiceConfigRead> {
        match self {
            ServiceConfig::Literal(config_str) => {
                let config = std::io::Cursor::new(config_str.clone());
                Ok(ServiceConfigRead::Literal { config })
            }
            ServiceConfig::File { path } => {
                let file = File::open(path).await?;
                Ok(ServiceConfigRead::File { file })
            }
        }
    }
}

pin_project_lite::pin_project! {
    #[project = ServiceConfigReadProj]
    pub enum ServiceConfigRead {
        Literal { #[pin] config: std::io::Cursor<String> },
        File { #[pin] file: File },
    }
}

impl AsyncRead for ServiceConfigRead {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.project();
        match this {
            ServiceConfigReadProj::Literal { config } => config.poll_read(cx, buf),
            ServiceConfigReadProj::File { file } => file.poll_read(cx, buf),
        }
    }
}

pub struct FsConfigServiceInner {
    pub file: tokio::fs::File,
    pub memory_config: Config,
}

pub struct FsConfigService {
    pub inner: Arc<RwLock<FsConfigServiceInner>>,
}


#[derive(Debug, thiserror::Error)]
pub enum FsConfigServiceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Service Not found")]
    ServiceNotFound,
}

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
