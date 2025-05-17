use std::{
    collections::BTreeSet,
    net::{IpAddr, SocketAddr},
};

use crate::descriptor::ServiceDescriptor;

pub struct Item {
    pub bind: SocketAddr,
    pub service: ServiceDescriptor,
    pub description: Option<String>,
}

pub struct ItemQuery {
    pub bind_ip: Option<IpAddr>,
    pub bind_port: Option<u16>,
    pub service_name: Option<String>,
    pub tags: Option<BTreeSet<String>>,
    pub id: Option<String>
}
