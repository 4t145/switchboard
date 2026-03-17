use std::sync::Arc;

#[derive(
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct OutboundName(Arc<str>);

impl OutboundName {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self(name.into())
    }
}

#[derive(
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct OutboundEndpoint {
    pub host: String,
    pub port: u16,
    pub weight: Option<u32>,
}

impl OutboundEndpoint {
    pub fn socket_addr(&self) -> (&str, u16) {
        (&self.host, self.port)
    }
}

pub type OutboundMap = std::collections::HashMap<OutboundName, OutboundEndpoint>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(untagged)]
pub enum Outbound {
    Single(OutboundEndpoint),
    NamedMap(OutboundMap),
}
