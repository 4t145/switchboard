use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;
use switchboard_custom_config::{SerdeValue, SerdeValueError};

pub mod fs;
pub mod k8s;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default, Hash, PartialEq, Eq)]
pub struct ResolveConfig {
    #[serde(default)]
    pub fs: fs::FsResolveConfig,
    #[serde(default)]
    pub k8s: k8s::K8sResolveConfig,
}

pub struct ResolveConfigRequest {
    pub provider: String,
    pub config: SerdeValue,
}

pub trait ServiceConfigResolver: Send + Sync + 'static {
    fn resolve(
        &self,
        resolve_config: SerdeValue,
        context: crate::ControllerContext,
    ) -> BoxFuture<'_, Result<switchboard_model::ServiceConfig, ResolveServiceConfigError>>;
}

pub type SharedServiceConfigResolver = Arc<dyn ServiceConfigResolver>;

pub struct ServiceConfigResolverItem {
    pub meta: ServiceConfigResolverMeta,
    pub resolver: SharedServiceConfigResolver,
}

impl std::fmt::Debug for ServiceConfigResolverItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceConfigResolverItem")
            .field("meta", &self.meta)
            .finish()
    }
}

#[derive(Debug)]
pub struct ServiceConfigResolverMeta {
    pub name: String,
    pub description: Option<String>,
}
#[derive(Debug, Default)]
pub struct ServiceConfigResolverRegistry {
    pub resolvers: HashMap<String, ServiceConfigResolverItem>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveServiceConfigError {
    #[error("resolver not found: {0}")]
    ResolverNotFound(String),
    #[error("resolve error: {0}")]
    ResolveError(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("deserialize config error: {0}")]
    DeserializeConfigError(#[from] SerdeValueError),
}

impl ResolveServiceConfigError {
    pub fn resolve_error<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        ResolveServiceConfigError::ResolveError(Box::new(err))
    }
}

impl ServiceConfigResolverRegistry {
    pub fn new() -> Self {
        Self {
            resolvers: HashMap::new(),
        }
    }
    pub fn register_resolver(
        &mut self,
        meta: ServiceConfigResolverMeta,
        resolver: SharedServiceConfigResolver,
    ) {
        let name = meta.name.clone();
        let item = ServiceConfigResolverItem { meta, resolver };
        self.resolvers.insert(name, item);
    }
    pub fn prelude() -> Self {
        let mut registry = Self::new();
        registry.register_resolver(
            ServiceConfigResolverMeta {
                name: "fs".to_string(),
                description: Some("Filesystem-based ServiceConfig resolver".to_string()),
            },
            Arc::new(fs::FsServiceConfigResolver),
        );
        registry.register_resolver(
            ServiceConfigResolverMeta {
                name: "k8s".to_string(),
                description: Some("Kubernetes-based ServiceConfig resolver".to_string()),
            },
            Arc::new(k8s::K8sServiceConfigResolver),
        );
        registry
    }
    pub fn list_resolvers(&self) -> Vec<&ServiceConfigResolverMeta> {
        self.resolvers
            .values()
            .map(|item| &item.meta)
            .collect::<Vec<_>>()
    }
    pub async fn resolve(
        &self,
        resolver: &str,
        config: SerdeValue,
        context: crate::ControllerContext,
    ) -> Result<switchboard_model::ServiceConfig, ResolveServiceConfigError> {
        if let Some(item) = self.resolvers.get(resolver) {
            item.resolver.resolve(config, context).await
        } else {
            Err(ResolveServiceConfigError::ResolverNotFound(
                resolver.to_string(),
            ))
        }
    }
}

impl crate::ControllerContext {
    pub async fn resolve_config(
        &self,
        resolver: &str,
        config: SerdeValue,
    ) -> Result<switchboard_model::ServiceConfig, crate::Error> {
        let config = self.resolve.resolve(resolver, config, self.clone()).await?;
        Ok(config)
    }

    pub async fn resolve_config_from_fs(
        &self,
    ) -> Result<switchboard_model::ServiceConfig, crate::Error> {
        self.resolve_config("fs", SerdeValue::default()).await
    }
}
