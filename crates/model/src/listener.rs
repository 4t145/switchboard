use std::{
    collections::BTreeSet,
    net::{IpAddr, SocketAddr},
};

use serde::{Deserialize, Serialize};

#[derive(
    Debug,
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
#[builder(on(String, into))]
pub struct Listener {
    pub bind: SocketAddr,
    pub description: Option<String>,
}

impl std::fmt::Display for Listener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.bind)?;
        if let Some(description) = &self.description {
            write!(f, " ({})", description)
        } else {
            Ok(())
        }
    }
}

pub struct BindQuery {
    pub bind_ip: Option<IpAddr>,
    pub bind_port: Option<u16>,
    pub service_name: Option<String>,
    pub tags: Option<BTreeSet<String>>,
    pub id: Option<String>,
}
