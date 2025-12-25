use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
#[cfg(feature = "fs")]
mod fs;
pub use fs::FsLinkResolver;
mod k8s;
pub use k8s::{K8sResource, K8sResourceParseError};
mod env;
pub use env::EnvLinkResolver;
use crate::{ConfigWithFormat, formats::TransferObject};

#[derive(Debug, thiserror::Error)]
pub enum LinkParseError {
    #[error("Invalid link format")]
    InvalidFormat,
    #[error("Unsupported link scheme: {scheme}")]
    UnsupportedScheme { scheme: String },
    #[error(transparent)]
    InvalidK8sResource(#[from] k8s::K8sResourceParseError),
}

#[derive(Debug, Clone, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub enum Link {
    Fs { path: PathBuf },
    Http { url: String },
    K8s(K8sResource),
    Env(String),
}

impl From<PathBuf> for Link {
    fn from(path: PathBuf) -> Self {
        Link::Fs { path }
    }
}

impl From<&PathBuf> for Link {
    fn from(path: &PathBuf) -> Self {
        Link::Fs { path: path.clone() }
    }
}

impl From<&Path> for Link {
    fn from(path: &Path) -> Self {
        Link::Fs {
            path: path.to_path_buf(),
        }
    }
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

impl std::fmt::Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Link::Fs { path } => write!(f, "file://{}", path.to_string_lossy()),
            Link::Http { url } => write!(f, "{}", url),
            Link::K8s(k8s_resource) => {
                write!(f, "k8s://{}", k8s_resource)
            }
            Link::Env(var_name) => {
                write!(f, "env://{}", var_name)
            }
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
                "k8s" => Ok(Link::K8s(K8sResource::from_str(rest)?)),
                "env" => Ok(Link::Env(rest.to_string())),
                _ => Err(LinkParseError::UnsupportedScheme {
                    scheme: scheme.to_string(),
                }),
            }
        } else {
            Err(LinkParseError::InvalidFormat)
        }
    }
}

impl Link {
    pub fn is_file(&self) -> bool {
        matches!(self, Link::Fs { .. })
    }
    pub fn is_http_resource(&self) -> bool {
        matches!(self, Link::Http { .. })
    }
    pub fn is_k8s_resource(&self) -> bool {
        matches!(self, Link::K8s { .. })
    }
    pub fn is_env_variable(&self) -> bool {
        matches!(self, Link::Env { .. })
    }
    pub fn as_file_path(&self) -> Option<&Path> {
        match self {
            Link::Fs { path } => Some(path),
            _ => None,
        }
    }
    pub fn as_http_url(&self) -> Option<&str> {
        match self {
            Link::Http { url } => Some(url),
            _ => None,
        }
    }
    pub fn as_k8s_resource(&self) -> Option<&K8sResource> {
        match self {
            Link::K8s(k8s_resource) => Some(k8s_resource),
            _ => None,
        }
    }
    pub fn as_env_ame(&self) -> Option<&str> {
        match self {
            Link::Env(var_name) => Some(var_name),
            _ => None,
        }
    }
}

pub trait LinkResolver: Clone + Send + Sync + 'static {
    fn fetch<V: TransferObject>(
        &self,
        link: &Link,
    ) -> impl Future<Output = Result<ConfigWithFormat<V>, crate::Error>> + Send;
}
