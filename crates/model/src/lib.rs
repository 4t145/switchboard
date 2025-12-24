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

impl<ServiceConfig> Config<ServiceConfig, crate::tls::TlsResolverInFile> {
    pub async fn resolve_tls_with_skip(self) -> Config<ServiceConfig, crate::tls::TlsResolver> {
        let mut resolved_tls = BTreeMap::new();
        let mut task_set = tokio::task::JoinSet::<
            Result<(String, Tls<crate::tls::TlsResolver>), crate::tls::TlsResolverLoadError>,
        >::new();
        for (name, tls_in_file) in self.tls.clone().into_iter() {
            task_set.spawn(async move {
                let resolver = tls_in_file.resolver.resolve_from_fs().await?;
                let tls = crate::tls::Tls {
                    resolver,
                    options: tls_in_file.options,
                };
                Ok((name, tls))
            });
        }
        while let Some(res) = task_set.join_next().await {
            match res {
                Ok(Ok((name, tls))) => {
                    resolved_tls.insert(name, tls);
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to resolve TLS: {}", e);
                }
                Err(e) => {
                    tracing::error!("TLS resolve task join error: {}", e);
                }
            }
        }
        Config {
            tcp_services: self.tcp_services,
            tcp_listeners: self.tcp_listeners,
            tcp_routes: self.tcp_routes,
            tls: resolved_tls,
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
