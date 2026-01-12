use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use switchboard_custom_config::{LinkOrValue, LinkResolver};

use crate::bytes::Base64Bytes;
#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]

pub struct Tls<TlsResolver = self::TlsResolver> {
    pub resolver: TlsResolver,
    pub options: TlsOptions,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]

#[serde(tag = "type", content = "data")]
pub enum TlsResolver {
    Single(TlsCertParams),
    Sni(BTreeMap<String, TlsCertParams>),
}

impl From<TlsCertParams> for TlsResolver {
    fn from(params: TlsCertParams) -> Self {
        TlsResolver::Single(params)
    }
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
    pub fn from_bytes(cert_bytes: &[u8], key_bytes: &[u8]) -> Result<Self, TlsCertParamsError> {
        let mut certs: Vec<Base64Bytes> = Vec::new();
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
    pub fn from_resolved_pem_files(
        certs: PemsFile,
        key: PemFile,
    ) -> Result<Self, TlsCertParamsError> {
        let certs: Vec<Base64Bytes> = certs
            .0
            .into_iter()
            .map(|pem| Base64Bytes(pem.into_contents()))
            .collect();
        let key = Base64Bytes(key.0.into_contents());
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
#[derive(Debug, Clone, PartialEq)]
pub struct PemFile(pub pem::Pem);

impl PemFile {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, pem::PemError> {
        let pem = pem::parse(bytes)?;
        Ok(PemFile(pem))
    }
}

impl FromStr for PemFile {
    type Err = pem::PemError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pem = pem::parse(s)?;
        Ok(PemFile(pem))
    }
}

impl Display for PemFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = pem::encode(&self.0);
        write!(f, "{}", s)
    }
}


impl bincode::Encode for PemFile {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let s = pem::encode(&self.0);
        s.encode(encoder)
    }
}

impl<C> bincode::Decode<C> for PemFile {
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let s = String::decode(decoder)?;
        let pem =
            pem::parse(s).map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))?;
        Ok(PemFile(pem))
    }
}

impl<'de, C> bincode::de::BorrowDecode<'de, C> for PemFile {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let s = String::borrow_decode(decoder)?;
        let pem =
            pem::parse(s).map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))?;
        Ok(PemFile(pem))
    }
}

impl Serialize for PemFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = pem::encode(&self.0);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for PemFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let pem = pem::parse(s).map_err(serde::de::Error::custom)?;
        Ok(PemFile(pem))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PemsFile(pub Vec<pem::Pem>);

impl PemsFile {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, pem::PemError> {
        let pems = pem::parse_many(bytes)?;
        Ok(PemsFile(pems))
    }
}

impl FromStr for PemsFile {
    type Err = pem::PemError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pems = pem::parse_many(s)?;
        Ok(PemsFile(pems))
    }
}

impl Display for PemsFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = pem::encode_many(&self.0);
        write!(f, "{}", s)
    }
}

impl bincode::Encode for PemsFile {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let s = pem::encode_many(&self.0);
        s.encode(encoder)
    }
}

impl<C> bincode::Decode<C> for PemsFile {
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let s = String::decode(decoder)?;
        let pems = pem::parse_many(s)
            .map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))?;
        Ok(PemsFile(pems))
    }
}

impl<'de, C> bincode::de::BorrowDecode<'de, C> for PemsFile {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let s = String::borrow_decode(decoder)?;
        let pem = pem::parse_many(s)
            .map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))?;
        Ok(PemsFile(pem))
    }
}

impl Serialize for PemsFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = pem::encode_many(&self.0);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for PemsFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let pem = pem::parse_many(s).map_err(serde::de::Error::custom)?;
        Ok(PemsFile(pem))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct FileStyleTlsResolverItem {
    #[serde(alias = "cert")]
    pub certs: LinkOrValue<PemsFile>,
    pub key: LinkOrValue<PemFile>,
}

impl FileStyleTlsResolverItem {
    pub async fn resolve<R: LinkResolver>(
        self,
        resolver: &R,
    ) -> Result<TlsCertParams, TlsResolverLoadError> {
        let certs = self.certs.resolve(resolver).await?;
        let key = self.key.resolve(resolver).await?;
        let params = TlsCertParams::from_resolved_pem_files(certs, key)?;
        Ok(params)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct TlsResolverItemInFileWithHostname {
    #[serde(alias = "domain")]
    pub hostname: String,
    #[serde(flatten)]
    pub tls_in_file: FileStyleTlsResolverItem,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(untagged)]
pub enum FileStyleTlsResolver {
    Single(FileStyleTlsResolverItem),
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
    #[error("Resolve error: {0}")]
    ResolveError(#[from] switchboard_custom_config::Error),
}

impl FileStyleTlsResolver {
    pub async fn resolve<R: LinkResolver>(
        self,
        resolver: &R,
    ) -> Result<TlsResolver, TlsResolverLoadError> {
        match self {
            FileStyleTlsResolver::Single(tls_in_file) => {
                let params = tls_in_file.resolve(resolver).await?;

                Ok(TlsResolver::Single(params))
            }
            FileStyleTlsResolver::Sni { sni: tls_files } => {
                let mut map = BTreeMap::new();
                for tls_in_file_with_hostname in tls_files.into_iter() {
                    let params = tls_in_file_with_hostname
                        .tls_in_file
                        .resolve(resolver)
                        .await?;
                    map.insert(tls_in_file_with_hostname.hostname.clone(), params);
                }
                Ok(TlsResolver::Sni(map))
            }
        }
    }
}

impl<ServiceConfig> crate::ServiceConfig<ServiceConfig, crate::tls::FileStyleTlsResolver> {
    pub async fn resolve_tls_with_skip<R: LinkResolver>(
        self,
        resolver: &R,
    ) -> crate::ServiceConfig<ServiceConfig, crate::tls::TlsResolver> {
        let mut resolved_tls = BTreeMap::new();
        let mut task_set = tokio::task::JoinSet::<
            Result<(String, Tls<crate::tls::TlsResolver>), crate::tls::TlsResolverLoadError>,
        >::new();
        for (name, tls_in_file) in self.tls.clone().into_iter() {
            let resolver = resolver.clone();
            task_set.spawn(async move {
                let resolver = tls_in_file.resolver.resolve(&resolver).await?;
                let tls = crate::tls::Tls {
                    resolver,
                    options: tls_in_file.options,
                };
                Ok((name, tls))
            });
        }
        while let Some(res) = task_set.join_next().await {
            match res {
                Ok(Ok((name, tls))) => {
                    resolved_tls.insert(name, tls);
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to resolve TLS: {}", e);
                }
                Err(e) => {
                    tracing::error!("TLS resolve task join error: {}", e);
                }
            }
        }
        crate::ServiceConfig {
            tcp_services: self.tcp_services,
            tcp_listeners: self.tcp_listeners,
            tcp_routes: self.tcp_routes,
            tls: resolved_tls,
        }
    }
}
