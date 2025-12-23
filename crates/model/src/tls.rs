use std::{borrow::Cow, collections::BTreeMap, fmt::Debug, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::bytes::Base64Bytes;
#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
pub struct Tls<TlsResolver = self::TlsResolver> {
    pub resolver: TlsResolver,
    pub options: TlsOptions,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "data")]
pub enum TlsResolver {
    Single(TlsCertParams),
    Sni(BTreeMap<String, TlsCertParams>),
}

#[derive(
    Clone,
    bon::Builder,
    Serialize,
    Deserialize,
    Hash,
    bincode::Encode,
    bincode::Decode,
    PartialEq,
    Eq,
)]
#[serde(rename_all = "camelCase")]
pub struct TlsCertParams {
    pub certs: Vec<Base64Bytes>,
    pub key: Base64Bytes,
    pub ocsp: Option<Base64Bytes>,
}

#[derive(Debug, thiserror::Error)]
pub enum TlsCertParamsError {
    #[error("Fail to parse certs file: {0}")]
    CertParseError(pem::PemError),
    #[error("Fail to parse key file: {0}")]
    KeyParseError(pem::PemError),
}
impl TlsCertParams {
    pub fn from_pem_file(cert_bytes: &[u8], key_bytes: &[u8]) -> Result<Self, TlsCertParamsError> {
        let mut certs = Vec::new();
        for pem in pem::parse_many(cert_bytes).map_err(TlsCertParamsError::CertParseError)? {
            let bytes = pem.into_contents();
            certs.push(Base64Bytes(bytes));
        }
        let key = pem::parse(key_bytes)
            .map_err(TlsCertParamsError::KeyParseError)?
            .into_contents();
        let key = Base64Bytes(key);
        Ok(TlsCertParams {
            certs,
            key,
            ocsp: None,
        })
    }
}

impl Debug for TlsCertParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsCertParams")
            .field("cert_chain_length", &self.certs.len())
            .finish()
    }
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    bon::Builder,
    Hash,
    bincode::Encode,
    bincode::Decode,
    PartialEq,
    Eq,
)]
#[builder(on(String, into))]
pub struct TlsOptions {
    #[builder(default)]
    pub ignore_client_order: bool,
    pub max_fragment_size: Option<u32>,
    #[builder(default)]
    pub alpn_protocols: Vec<String>,
    #[builder(default)]
    pub enable_secret_extraction: bool,
    #[builder(default)]
    pub max_early_data_size: u32,
    #[builder(default)]
    pub send_half_rtt_data: bool,
    #[builder(default)]
    pub send_tls13_tickets: u32,
    #[builder(default)]
    pub require_ems: bool,
}

impl Default for TlsOptions {
    fn default() -> Self {
        TlsOptions {
            ignore_client_order: false,
            max_fragment_size: None,
            alpn_protocols: Vec::new(),
            enable_secret_extraction: false,
            max_early_data_size: 0,
            send_half_rtt_data: false,
            send_tls13_tickets: 2,
            require_ems: true,
        }
    }
}

#[derive(Debug, Clone, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub enum EmbeddedOrFilePem {
    Embedded(String),
    File(std::path::PathBuf),
}

impl EmbeddedOrFilePem {
    pub fn path(&self) -> Option<&std::path::Path> {
        match self {
            EmbeddedOrFilePem::Embedded(_) => None,
            EmbeddedOrFilePem::File(path) => Some(path.as_path()),
        }
    }
}

impl Serialize for EmbeddedOrFilePem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            EmbeddedOrFilePem::Embedded(s) => serializer.serialize_str(s),
            EmbeddedOrFilePem::File(path) => {
                let s = format!("file://{}", path.display());
                serializer.serialize_str(&s)
            }
        }
    }
}

impl<'de> Deserialize<'de> for EmbeddedOrFilePem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        EmbeddedOrFilePem::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for EmbeddedOrFilePem {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("file://") {
            let path = &s[7..];
            Ok(EmbeddedOrFilePem::File(std::path::PathBuf::from(path)))
        } else {
            Ok(EmbeddedOrFilePem::Embedded(s.to_string()))
        }
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
pub struct TlsResolverItemInFile {
    #[serde(alias = "cert")]
    pub certs: EmbeddedOrFilePem,
    pub key: EmbeddedOrFilePem,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
pub struct TlsResolverItemInFileWithHostname {
    #[serde(alias = "domain")]
    pub hostname: String,
    #[serde(flatten)]
    pub tls_in_file: TlsResolverItemInFile,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
#[serde(untagged)]
pub enum TlsResolverInFile {
    Single(TlsResolverItemInFile),
    Sni {
        sni: Vec<TlsResolverItemInFileWithHostname>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum TlsResolverLoadError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TLS cert params error: {0}")]
    TlsCertParamsError(#[from] TlsCertParamsError),
}
impl TlsResolverInFile {
    async fn resolve_to_bytes(pem: &EmbeddedOrFilePem) -> Result<Cow<'_, [u8]>, std::io::Error> {
        match pem {
            EmbeddedOrFilePem::Embedded(s) => Ok(s.as_bytes().into()),
            EmbeddedOrFilePem::File(path) => {
                let bytes = tokio::fs::read(path).await?;
                Ok(bytes.into())
            }
        }
    }
}
impl TlsResolverItemInFile {
    pub async fn resolve_from_fs(&self) -> Result<TlsCertParams, TlsResolverLoadError> {
        if let Some(path) = self.certs.path() {
            tracing::debug!("Loading TLS certs from file: {}", path.display());
        }
        let certs_bytes = TlsResolverInFile::resolve_to_bytes(&self.certs).await?;
        if let Some(path) = self.key.path() {
            tracing::debug!("Loading TLS key from file: {}", path.display());
        }
        let key_bytes = TlsResolverInFile::resolve_to_bytes(&self.key).await?;
        let params = TlsCertParams::from_pem_file(&certs_bytes, &key_bytes)?;
        Ok(params)
    }
}

impl TlsResolverInFile {
    pub async fn resolve_from_fs(&self) -> Result<TlsResolver, TlsResolverLoadError> {
        match self {
            TlsResolverInFile::Single(tls_in_file) => {
                let params = tls_in_file.resolve_from_fs().await?;
                Ok(TlsResolver::Single(params))
            }
            TlsResolverInFile::Sni { sni: tls_files } => {
                let mut map = BTreeMap::new();
                for tls_in_file_with_hostname in tls_files.iter() {
                    let params = tls_in_file_with_hostname
                        .tls_in_file
                        .resolve_from_fs()
                        .await?;
                    map.insert(tls_in_file_with_hostname.hostname.clone(), params);
                }
                Ok(TlsResolver::Sni(map))
            }
        }
    }
}
