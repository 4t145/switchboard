use futures::future::BoxFuture;
use switchboard_custom_config::SerdeValue;

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
        config: SerdeValue,
    ) -> BoxFuture<'_, Result<switchboard_model::Config, Box<dyn std::error::Error + Send + Sync>>>;
}
