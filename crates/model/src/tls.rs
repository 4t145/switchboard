use std::{collections::BTreeMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::bytes::Base64Bytes;
#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
pub struct Tls {
    pub resolver: TlsResolver,
    pub options: TlsOptions,
}
#[derive(
    Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
pub enum TlsResolver {
    Sni(BTreeMap<String, TlsCertParams>),
    Single(TlsCertParams),
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
