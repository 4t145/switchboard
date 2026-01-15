use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
// use switchboard_custom_config::{LinkOrValue, LinkResolver};
use switchboard_link_or_value::{
    LinkOrValue, Resolvable, Resolver, resolver::string_parse::StringParseResolver,
};

use crate::bytes::Base64Bytes;
#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]

pub struct Tls<TlsResolver = self::TlsResolver> {
    pub resolver: TlsResolver,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<TlsOptions>,
}

impl<L, TlsResolver> Resolvable<L, TlsResolver, Tls<TlsResolver>>
    for Tls<LinkOrValue<L, TlsResolver>>
where
    L: Send + Sync + 'static,
    TlsResolver: Send + Sync + 'static,
{
    async fn resolve_with<R: Resolver<L, TlsResolver>>(
        self,
        resolver: &R,
    ) -> Result<Tls<TlsResolver>, R::Error> {
        let resolved_resolver = self.resolver.resolve_with(resolver).await?;
        Ok(Tls {
            resolver: resolved_resolver,
            options: self.options,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq)]
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
    Clone, bon::Builder, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq,
)]

pub struct TlsCertParams<C = PemsFile, K = PemFile> {
    #[serde(alias = "cert")]
    pub certs: C,
    pub key: K,
    #[serde(skip_serializing_if = "Option::is_none", default)]
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
        let mut certs: PemsFile = PemsFile(Vec::new());
        for pem in pem::parse_many(cert_bytes).map_err(TlsCertParamsError::CertParseError)? {
            certs.0.push(pem.into());
        }
        let key = pem::parse(key_bytes)
            .map_err(TlsCertParamsError::KeyParseError)?
            .into();
        Ok(TlsCertParams {
            certs,
            key,
            ocsp: None,
        })
    }
    pub fn from_resolved_pem_files(certs: PemsFile, key: PemFile) -> Self {
        TlsCertParams {
            certs,
            key,
            ocsp: None,
        }
    }
}

impl<C, K> Debug for TlsCertParams<C, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsCertParams").finish()
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

impl From<pem::Pem> for PemFile {
    fn from(pem: pem::Pem) -> Self {
        PemFile(pem)
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

impl From<Vec<pem::Pem>> for PemsFile {
    fn from(pems: Vec<pem::Pem>) -> Self {
        PemsFile(pems)
    }
}

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
pub struct FileStyleTlsResolverItem<C = PemsFile, K = PemFile> {
    #[serde(alias = "cert")]
    pub certs: C,
    pub key: K,
}

impl<L, C, K> Resolvable<L, C, FileStyleTlsResolverItem<C, K>>
    for FileStyleTlsResolverItem<LinkOrValue<L, C>, K>
where
    L: Send + Sync + 'static,
    C: Send + Sync + 'static,
    K: Send + Sync + 'static,
{
    async fn resolve_with<R: Resolver<L, C>>(
        self,
        resolver: &R,
    ) -> Result<FileStyleTlsResolverItem<C, K>, R::Error> {
        let resolved_certs = self.certs.resolve_with(resolver).await?;
        Ok(FileStyleTlsResolverItem {
            certs: resolved_certs,
            key: self.key,
        })
    }
}

impl<L, C, K> Resolvable<L, K, FileStyleTlsResolverItem<C, K>>
    for FileStyleTlsResolverItem<C, LinkOrValue<L, K>>
where
    L: Send + Sync + 'static,
    C: Send + Sync + 'static,
    K: Send + Sync + 'static,
{
    async fn resolve_with<R: Resolver<L, K>>(
        self,
        resolver: &R,
    ) -> Result<FileStyleTlsResolverItem<C, K>, R::Error> {
        let resolved_key = self.key.resolve_with(resolver).await?;
        Ok(FileStyleTlsResolverItem {
            certs: self.certs,
            key: resolved_key,
        })
    }
}
impl FileStyleTlsResolverItem {
    pub fn into_pem_files(self) -> TlsCertParams {
        TlsCertParams::from_resolved_pem_files(self.certs, self.key)
    }
}

// impl FileStyleTlsResolverItem<Pem
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct TlsResolverItemInFileWithHostname<C = PemsFile, K = PemFile> {
    #[serde(alias = "domain")]
    pub hostname: String,
    #[serde(flatten)]
    pub tls_in_file: TlsCertParams<C, K>,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(untagged)]
pub enum FileStyleTlsResolver<C = PemsFile, K = PemFile> {
    Single(TlsCertParams<C, K>),
    Sni {
        sni: Vec<TlsResolverItemInFileWithHostname<C, K>>,
    },
}
pub type UnresolvedFileStyleTlsResolver<L> =
    FileStyleTlsResolver<LinkOrValue<L, PemsFile>, LinkOrValue<L, PemFile>>;
#[derive(Debug, thiserror::Error)]
pub enum TlsResolveError {
    #[error("Failed to resolve certs: {source}")]
    ResolveCertsError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("Failed to resolve key: {source}")]
    ResolveKeyError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl FileStyleTlsResolver {
    pub fn into_standard(self) -> TlsResolver {
        match self {
            FileStyleTlsResolver::Single(tls_in_file) => TlsResolver::Single(tls_in_file),
            FileStyleTlsResolver::Sni { sni: tls_files } => {
                let mut map = BTreeMap::new();
                for tls_in_file_with_hostname in tls_files.into_iter() {
                    let params = tls_in_file_with_hostname.tls_in_file;
                    map.insert(tls_in_file_with_hostname.hostname.clone(), params);
                }
                TlsResolver::Sni(map)
            }
        }
    }
}

impl<L> UnresolvedFileStyleTlsResolver<L> {
    pub async fn resolve_to_standard<R>(self, resolver: &R) -> Result<TlsResolver, TlsResolveError>
    where
        R: Resolver<L, String>,
        L: Send + Sync + 'static,
    {
        let str_resolver = StringParseResolver::new(resolver.clone());
        match self {
            FileStyleTlsResolver::Single(unresolved_cert_params) => {
                let certs = unresolved_cert_params
                    .certs
                    .resolve_with(&str_resolver)
                    .await
                    .map_err(|e| TlsResolveError::ResolveCertsError {
                        source: Box::new(e),
                    })?;
                let key = unresolved_cert_params
                    .key
                    .resolve_with(&str_resolver)
                    .await
                    .map_err(|e| TlsResolveError::ResolveKeyError {
                        source: Box::new(e),
                    })?;
                let tls_params = TlsCertParams::from_resolved_pem_files(certs, key);
                let resolver = crate::tls::TlsResolver::Single(tls_params);

                return Ok(resolver);
            }
            FileStyleTlsResolver::Sni { sni } => {
                let mut sni_map = BTreeMap::new();
                for item in sni.into_iter() {
                    let certs = item
                        .tls_in_file
                        .certs
                        .resolve_with(&str_resolver)
                        .await
                        .map_err(|e| TlsResolveError::ResolveCertsError {
                            source: Box::new(e),
                        })?;
                    let key = item
                        .tls_in_file
                        .key
                        .resolve_with(&str_resolver)
                        .await
                        .map_err(|e| TlsResolveError::ResolveKeyError {
                            source: Box::new(e),
                        })?;
                    let tls_params = TlsCertParams::from_resolved_pem_files(certs, key);
                    sni_map.insert(item.hostname.clone(), tls_params);
                }
                let resolver = crate::tls::TlsResolver::Sni(sni_map);
                return Ok(resolver);
            }
        }
    }
}
