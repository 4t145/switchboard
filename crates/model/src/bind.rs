use std::{
    collections::BTreeSet,
    net::{IpAddr, SocketAddr},
};

use crate::descriptor::ServiceDescriptor;
#[derive(Debug, Clone, bon::Builder)]
#[builder(on(String, into))]
pub struct Bind {
    pub addr: SocketAddr,
    #[builder(into)]
    pub service: ServiceDescriptor,
    pub description: Option<String>,
}

impl std::fmt::Display for Bind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.addr, self.service)?;
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
