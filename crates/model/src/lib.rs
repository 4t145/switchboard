use std::{collections::BTreeMap, net::SocketAddr};
pub const MODEL_VERSION: &str = env!("CARGO_PKG_VERSION");
pub mod cursor;
pub use cursor::*;
pub mod descriptor;
pub use descriptor::*;
pub mod listener;
pub use listener::*;
pub mod tag;
use serde::{Deserialize, Serialize};
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
pub enum ConfigEvent {
    Reload,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct Config<Cfg = switchboard_custom_config::CustomConfig> {
    pub tcp_services: BTreeMap<String, TcpServiceConfig<Cfg>>,
    pub tcp_listeners: BTreeMap<SocketAddr, Listener>,
    pub tcp_routes: BTreeMap<SocketAddr, TcpRoute>,
    pub tls: BTreeMap<String, Tls>,
}

impl Config {
    pub fn get_tcp_service(&self, name: &str) -> Option<&TcpServiceConfig> {
        self.tcp_services.get(name)
    }

    pub fn get_tls(&self, name: &str) -> Option<&Tls> {
        self.tls.get(name)
    }
}

pub trait ConfigService {
    type Error: std::error::Error;
    fn fetch_config(&self) -> impl Future<Output = Result<Config, Self::Error>> + Send + '_;
}

pub trait ResourceService {}
