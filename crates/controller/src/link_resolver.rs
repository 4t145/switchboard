use std::{fmt::Display, path::PathBuf, str::FromStr};

use http::Uri;
use switchboard_file_resolver::FileResolver;
use switchboard_link_or_value::Resolver;
use switchboard_model::SerdeValue;

use crate::storage::{InvalidStorageObjectDescriptor, StorageError, StorageObjectDescriptor};

#[derive(Clone)]
pub struct ControllerLinkResolver {
    context: crate::ControllerContext,
    file: FileResolver,
}

impl crate::ControllerContext {
    pub fn link_resolver(self) -> ControllerLinkResolver {
        ControllerLinkResolver {
            context: self,
            file: FileResolver::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkResolveError {
    #[error("Link parse error: {0}")]
    LinkParseError(#[from] LinkParseError),
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    #[error("File resolve error: {0}")]
    FileResolveError(#[from] switchboard_file_resolver::FileResolveError),
    #[error("Resource not found: {link}")]
    NotFound { link: Link },
    #[error("No implementation for link: {link}")]
    NoImplementation { link: Link },
}

impl Resolver<Link, String> for ControllerLinkResolver {
    type Error = LinkResolveError;
    async fn resolve(&self, link: Link) -> Result<String, Self::Error> {
        match &link {
            Link::FilePath(path) => {
                let string = self.file.resolve_string(path.clone()).await?;
                Ok(string)
            }
            Link::Storage(storage_object_descriptor) => {
                let storage = &self.context.storage;
                let object = storage
                    .get_object(storage_object_descriptor)
                    .await
                    .map_err(LinkResolveError::StorageError)?
                    .ok_or_else(|| LinkResolveError::NotFound { link })?;
                let serde_value: SerdeValue = object.data;
                let string = serde_value.deserialize_into::<String>().map_err(|e| {
                    LinkResolveError::FileResolveError(
                        switchboard_file_resolver::FileResolveError::from_deserialization_error(
                            e, "String",
                        ),
                    )
                })?;
                Ok(string)
            }
            _ => Err(LinkResolveError::NoImplementation { link }),
        }
    }
}

impl Resolver<Link, SerdeValue> for ControllerLinkResolver {
    type Error = LinkResolveError;
    async fn resolve(&self, link: Link) -> Result<SerdeValue, Self::Error> {
        match &link {
            Link::FilePath(path) => self
                .file
                .resolve_value(path.clone())
                .await
                .map_err(Into::into),
            Link::Storage(storage_object_descriptor) => {
                let storage = &self.context.storage;
                let object = storage
                    .get_object(storage_object_descriptor)
                    .await
                    .map_err(LinkResolveError::StorageError)?
                    .ok_or_else(|| LinkResolveError::NotFound { link })?;
                Ok(object.data)
            }
            _ => Err(LinkResolveError::NoImplementation { link }),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Link {
    FilePath(PathBuf),
    Http(Uri),
    Storage(StorageObjectDescriptor),
}

impl serde::Serialize for Link {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Link {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Link::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Link {
    pub fn file_path(path: impl Into<PathBuf>) -> Self {
        Link::FilePath(path.into())
    }
    pub fn http(uri: impl Into<Uri>) -> Self {
        Link::Http(uri.into())
    }
    pub fn storage(descriptor: StorageObjectDescriptor) -> Self {
        Link::Storage(descriptor)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkParseError {
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("Invalid storage descriptor")]
    InvalidStorageDescriptor(#[from] InvalidStorageObjectDescriptor),
    #[error("Other error: {0}")]
    Other(String),
}

impl FromStr for Link {
    type Err = LinkParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(path) = s.strip_prefix("file://") {
            Ok(Link::FilePath(PathBuf::from(path)))
        } else if let Some(descriptor_str) = s.strip_prefix("storage://") {
            // Here we would parse the storage descriptor properly.
            // For simplicity, we will just create a dummy descriptor.
            let descriptor = StorageObjectDescriptor::from_str(descriptor_str)?;
            Ok(Link::Storage(descriptor))
        } else {
            let uri: Uri = s.parse()?;
            Ok(Link::Http(uri))
        }
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Link::FilePath(path) => write!(f, "file://{}", path.display()),
            Link::Http(uri) => write!(f, "{}", uri),
            Link::Storage(descriptor) => write!(f, "storage://{}", descriptor),
        }
    }
}
