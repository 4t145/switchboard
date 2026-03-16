use std::{sync::Arc};

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct OutboundName(Arc<str>);

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct Outbound {
    pub endpoint: String,
    pub port: u16,
}

impl Outbound {
    pub fn socket_addr(&self) -> (&str, u16) {
        (&self.endpoint, self.port)
    }
}

