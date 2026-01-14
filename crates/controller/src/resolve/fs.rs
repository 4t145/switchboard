use std::path::PathBuf;
use switchboard_link_or_value::LinkOrValue;
use switchboard_model::{HumanReadableServiceConfig, SerdeValue};
pub use switchboard_model::resolve::file_style::*;

use crate::{
    link_resolver::{ControllerLinkResolver, Link},
    resolve::{ResolveServiceConfigError, ServiceConfigResolver},
};

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
        context: crate::ControllerContext,
    ) -> Result<HumanReadableServiceConfig<Link>, ResolveConfigFileError> {
        let svc_config = switchboard_model::resolve::file_style::fetch_human_readable_config(
            LinkOrValue::Link(Link::file_path(config.path)),
            &context.link_resolver(),
        )
        .await?;
        Ok(svc_config)
    }
}
impl ServiceConfigResolver for FsServiceConfigResolver {
    fn resolve(
        &self,
        resolve_config: SerdeValue,
        context: crate::ControllerContext,
    ) -> futures::future::BoxFuture<
        '_,
        Result<HumanReadableServiceConfig<Link>, ResolveServiceConfigError>,
    > {
        Box::pin(async move {
            let resolve_config = resolve_config.deserialize_into::<FsResolveConfig>()?;
            let config = self
                .resolve_service_config_from_fs(resolve_config, context)
                .await
                .map_err(ResolveServiceConfigError::resolve_error)?;
            Ok(config)
        })
    }
}
