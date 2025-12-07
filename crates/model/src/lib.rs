use std::collections::{BTreeMap, BTreeSet};
pub const MODEL_VERSION: &str = env!("CARGO_PKG_VERSION");
pub mod cursor;
pub use cursor::*;
pub mod descriptor;
pub use descriptor::*;
pub mod bind;
pub use bind::*;
pub mod tag;
use serde::{Deserialize, Serialize};
pub use tag::*;
pub mod named_service;
pub use named_service::*;
pub mod rbac;
pub mod tls;
pub use tls::*;
pub mod bytes;
pub mod control;
pub mod kernel;
pub mod controller;
pub mod protocol;
pub mod error;
pub enum ConfigEvent {
    Reload,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub named_services: BTreeMap<String, NamedService>,
    pub binds: BTreeMap<String, Bind>,
    pub enabled: BTreeSet<String>,
    pub tls: BTreeMap<String, Tls>,
}

impl Config {
    pub fn get_enabled(&self) -> impl Iterator<Item = (&str, &Bind)> + '_ {
        self.enabled
            .iter()
            .filter_map(|id| self.binds.get(id).map(|bind| (id.as_str(), bind)))
    }

    pub fn get_named_service(&self, name: &str) -> Option<&NamedService> {
        self.named_services.get(name)
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
