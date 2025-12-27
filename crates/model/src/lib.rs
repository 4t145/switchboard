use std::{collections::BTreeMap, net::SocketAddr};
pub const MODEL_VERSION: &str = env!("CARGO_PKG_VERSION");
pub use chrono;
pub mod resolve;
pub mod cursor;
pub use cursor::*;
pub mod descriptor;
pub use descriptor::*;
pub mod listener;
pub use listener::*;
pub mod tag;
use serde::{Deserialize, Serialize};
use switchboard_custom_config::SerdeValue;
pub use tag::*;
pub mod tcp_service;
pub use tcp_service::*;
pub mod rbac;
pub mod tls;
pub use tls::*;

use crate::tcp_route::TcpRoute;
pub mod bytes;
pub mod control;
pub mod controller;
pub mod error;
pub mod http;
pub mod kernel;
pub mod protocol;
pub mod regex;
pub mod services;
pub mod tcp_route;

pub use switchboard_custom_config as custom_config;
pub enum ConfigEvent {
    Reload,
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct Config<
    ServiceConfig = SerdeValue,
    TlsResolver = crate::tls::TlsResolver,
> {
    pub tcp_services: BTreeMap<String, TcpServiceConfig<ServiceConfig>>,
    pub tcp_listeners: BTreeMap<SocketAddr, Listener>,
    pub tcp_routes: BTreeMap<SocketAddr, TcpRoute>,
    pub tls: BTreeMap<String, Tls<TlsResolver>>,
}

impl<ServiceConfig, TlsResolver> Default for Config<ServiceConfig, TlsResolver> {
    fn default() -> Self {
        Config {
            tcp_services: BTreeMap::new(),
            tcp_listeners: BTreeMap::new(),
            tcp_routes: BTreeMap::new(),
            tls: BTreeMap::new(),
        }
    }
}


impl<ServiceConfig, TlsResolver> Config<ServiceConfig, TlsResolver> {
    pub fn get_tcp_service(&self, name: &str) -> Option<&TcpServiceConfig<ServiceConfig>> {
        self.tcp_services.get(name)
    }

    pub fn get_tls(&self, name: &str) -> Option<&Tls<TlsResolver>> {
        self.tls.get(name)
    }

}
