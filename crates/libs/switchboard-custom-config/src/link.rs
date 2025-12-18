use std::path::PathBuf;

use crate::CustomConfig;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
#[serde(transparent)]
pub struct Link(pub String);

impl From<PathBuf> for Link {
    fn from(path: PathBuf) -> Self {
        Link(format!("file://{}", path.to_string_lossy()))
    }
}   

pub trait LinkResolver {
    type Error: std::error::Error;
    fn fetch(&self, link: &Link) -> impl Future<Output = Result<CustomConfig, Self::Error>> + Send;
    fn upload(&self, link: &Link, config: &CustomConfig) -> impl Future<Output = Result<(), Self::Error>> + Send;
}