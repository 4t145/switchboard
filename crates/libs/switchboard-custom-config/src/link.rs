use std::{path::{Path, PathBuf}, str::FromStr};

use crate::CustomConfig;

impl From<PathBuf> for Link {
    fn from(path: PathBuf) -> Self {
        Link::Fs { path }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LinkParseError {
    #[error("Invalid link format")]
    InvalidFormat,
    #[error("Unsupported link scheme: {scheme}")]
    UnsupportedScheme { scheme: String },
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

impl ToString for Link {
    fn to_string(&self) -> String {
        match self {
            Link::Fs { path } => format!("file://{}", path.to_string_lossy()),
            Link::Http { url } => url.clone(),
        }
    }
}

impl FromStr for Link {
    type Err = LinkParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((scheme, rest)) = s.split_once("://") {
            match scheme {
                "file" => Ok(Link::Fs {
                    path: PathBuf::from(rest),
                }),
                "http" | "https" => Ok(Link::Http { url: s.to_string() }),
                _ => Err(LinkParseError::UnsupportedScheme {
                    scheme: scheme.to_string(),
                }),
            }
        } else {
            Err(LinkParseError::InvalidFormat)
        }
    }
}

#[derive(Debug, Clone, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub enum Link {
    Fs { path: PathBuf },
    Http { url: String },
}

impl Link {
    pub fn is_file(&self) -> bool {
        matches!(self, Link::Fs { .. })
    }
    pub fn as_file_path(&self) -> Option<&Path> {
        match self {
            Link::Fs { path } => Some(path),
            _ => None,
        }
    }
    pub fn is_http_resource(&self) -> bool {
        matches!(self, Link::Http { .. })
    }
}
pub trait LinkResolver {
    fn fetch(&self, link: &Link) -> impl Future<Output = Result<CustomConfig, crate::Error>> + Send;
    fn upload(
        &self,
        link: &Link,
        config: &CustomConfig,
    ) -> impl Future<Output = Result<(), crate::Error>> + Send;
}
