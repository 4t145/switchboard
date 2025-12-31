use std::path::PathBuf;
use switchboard_custom_config::{FsLinkResolver, Link, LinkOrValue};
pub use switchboard_model::resolve::fs::*;

use crate::resolve::{ResolveServiceConfigError, ServiceConfigResolver};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
#[serde(default)]
pub struct FsResolveConfig {
    pub path: PathBuf,
}

impl Default for FsResolveConfig {
    fn default() -> Self {
        Self {
            path: default_switchboard_config_path(),
        }
    }
}

pub struct FsServiceConfigResolver;

impl FsServiceConfigResolver {
    async fn resolve_service_config_from_fs(
        &self,
        config: FsResolveConfig,
    ) -> Result<switchboard_model::ServiceConfig, ResolveConfigFileError> {
        let svc_config = switchboard_model::resolve::fs::fetch_config(
            LinkOrValue::Link(Link::file_path(config.path)),
            &FsLinkResolver,
        )
        .await?;
        Ok(svc_config)
    }
}
impl ServiceConfigResolver for FsServiceConfigResolver {
    fn resolve(
        &self,
        resolve_config: switchboard_custom_config::SerdeValue,
        _context: crate::ControllerContext,
    ) -> futures::future::BoxFuture<
        '_,
        Result<switchboard_model::ServiceConfig, ResolveServiceConfigError>,
    > {
        Box::pin(async move {
            let resolve_config = resolve_config.deserialize_into::<FsResolveConfig>()?;
            let config = self
                .resolve_service_config_from_fs(resolve_config)
                .await
                .map_err(ResolveServiceConfigError::resolve_error)?;
            Ok(config)
        })
    }
}
