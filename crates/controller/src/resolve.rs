pub mod fs;
pub mod k8s;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default, Hash, PartialEq, Eq)]
pub struct ResolveConfig {
    #[serde(default)]
    pub fs: fs::FsResolveConfig,
    #[serde(default)]
    pub k8s: k8s::K8sResolveConfig,
}