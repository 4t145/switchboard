pub mod fs;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default, Hash, PartialEq, Eq)]
pub struct ResolveConfig {
    #[serde(default)]
    pub fs: fs::FsResolveConfig,
}