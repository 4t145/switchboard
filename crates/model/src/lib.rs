use std::{collections::BTreeMap, net::SocketAddr};
pub const MODEL_VERSION: &str = env!("CARGO_PKG_VERSION");
pub use chrono;
pub mod cursor;
pub mod resolve;
pub use cursor::*;
pub mod descriptor;
pub use descriptor::*;
pub mod listener;
pub use listener::*;
pub mod tag;
use serde::{Deserialize, Serialize};
use switchboard_custom_config::SerdeValue;
use switchboard_link_or_value::{LinkOrValue, Resolvable, Resolver};
pub use tag::*;
pub mod tcp_service;
pub use tcp_service::*;
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
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Hash, bincode::Encode, bincode::Decode,
)]
pub struct ServiceConfig<ConfigValue = SerdeValue, TlsResolver = crate::tls::TlsResolver> {
    pub tcp_services: BTreeMap<String, TcpServiceConfig<ConfigValue>>,
    pub tcp_listeners: BTreeMap<SocketAddr, Listener>,
    pub tcp_routes: BTreeMap<SocketAddr, TcpRoute>,
    pub tls: BTreeMap<String, Tls<TlsResolver>>,
}

impl<ConfigValue, TlsResolver> Default for ServiceConfig<ConfigValue, TlsResolver> {
    fn default() -> Self {
        ServiceConfig {
            tcp_services: BTreeMap::new(),
            tcp_listeners: BTreeMap::new(),
            tcp_routes: BTreeMap::new(),
            tls: BTreeMap::new(),
        }
    }
}

impl<ConfigValue, TlsResolver> ServiceConfig<ConfigValue, TlsResolver> {
    pub fn get_tcp_service(&self, name: &str) -> Option<&TcpServiceConfig<ConfigValue>> {
        self.tcp_services.get(name)
    }

    pub fn get_tls(&self, name: &str) -> Option<&Tls<TlsResolver>> {
        self.tls.get(name)
    }
}

impl<L, ConfigValue, TlsResolver>
    Resolvable<L, ConfigValue, ServiceConfig<ConfigValue, TlsResolver>>
    for ServiceConfig<LinkOrValue<L, ConfigValue>, TlsResolver>
where
    L: Send + Sync + 'static,
    ConfigValue: Send + Sync + 'static,
    TlsResolver: Send + Sync + 'static,
{
    async fn resolve_with<R: Resolver<L, ConfigValue> + ?Sized>(
        self,
        resolver: &R,
    ) -> Result<ServiceConfig<ConfigValue, TlsResolver>, R::Error> {
        let mut new_tcp_services = BTreeMap::new();
        for (name, tcp_service) in self.tcp_services.into_iter() {
            let resolved_tcp_service = tcp_service.resolve_with(resolver).await?;
            new_tcp_services.insert(name, resolved_tcp_service);
        }
        Ok(ServiceConfig {
            tcp_services: new_tcp_services,
            tcp_listeners: self.tcp_listeners,
            tcp_routes: self.tcp_routes,
            tls: self.tls,
        })
    }
}
