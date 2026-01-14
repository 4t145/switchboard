use std::{
    fs::File,
    path::{Path, PathBuf},
};

use ::serde::de::DeserializeOwned;
use switchboard_link_or_value::{Resolver, resolver::string_parse::StringParseResolver};
use switchboard_serde_value::SerdeValue;
#[derive(Debug, Clone, Copy, Default)]
pub struct FileResolver;
impl FileResolver {
    pub fn new() -> Self {
        Self
    }
    pub fn as_string_parser(&self) -> StringParseResolver<Self> {
        StringParseResolver::new(*self)
    }
    pub async fn resolve_string(&self, path: PathBuf) -> Result<String, FileResolveError> {
        <Self as Resolver<PathBuf, String>>::resolve(&self, path).await
    }
    pub async fn resolve_value(&self, path: PathBuf) -> Result<SerdeValue, FileResolveError> {
        <Self as Resolver<PathBuf, SerdeValue>>::resolve(&self, path).await
    }
}
impl Resolver<PathBuf, SerdeValue> for FileResolver {
    type Error = FileResolveError;
    async fn resolve(&self, path: PathBuf) -> Result<SerdeValue, Self::Error> {
        resolve_from_path(&path).await
    }
}

impl Resolver<PathBuf, String> for FileResolver {
    type Error = FileResolveError;
    async fn resolve(&self, path: PathBuf) -> Result<String, Self::Error> {
        let data = tokio::fs::read_to_string(&path).await?;
        Ok(data)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileResolveError {
    #[error("Unsupported file extension")]
    UnsupportedFileExtension,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Deserialization error {format}")]
    DeserializationError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        format: &'static str,
    },
}

impl FileResolveError {
    pub fn from_deserialization_error<E: std::error::Error + Send + Sync + 'static>(
        e: E,
        format: &'static str,
    ) -> Self {
        FileResolveError::DeserializationError {
            source: Box::new(e),
            format,
        }
    }
}

pub async fn resolve_from_path(path: &Path) -> Result<SerdeValue, FileResolveError> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or(FileResolveError::UnsupportedFileExtension)?;
    let data = tokio::fs::read(path).await?;
    match ext {
        "json" | "toml" | "toon" => deserialize_from_path::<SerdeValue>(path).await,
        "bincode" => {
            let (value, _) =
                bincode::decode_from_slice::<SerdeValue, _>(&data, bincode::config::standard())
                    .map_err(|e| FileResolveError::from_deserialization_error(e, "bincode"))?;
            Ok(value)
        }
        _ => Err(FileResolveError::UnsupportedFileExtension),
    }
}

pub async fn deserialize_from_path<T: DeserializeOwned>(
    path: &Path,
) -> Result<T, FileResolveError> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or(FileResolveError::UnsupportedFileExtension)?;
    let data = tokio::fs::read(path).await?;
    match ext {
        "json" => {
            let value: T = serde_json::from_slice(&data)
                .map_err(|e| FileResolveError::from_deserialization_error(e, "json"))?;
            Ok(value)
        }
        "toml" => {
            let value: T = toml::from_slice(&data)
                .map_err(|e| FileResolveError::from_deserialization_error(e, "toml"))?;
            Ok(value)
        }
        "toon" => {
            let value: T = serde_toon::from_slice(&data)
                .map_err(|e| FileResolveError::from_deserialization_error(e, "toon"))?;
            Ok(value)
        }
        _ => Err(FileResolveError::UnsupportedFileExtension),
    }
}
