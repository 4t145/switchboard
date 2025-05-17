use std::{collections::BTreeMap, path::PathBuf};

use switchboard_model::ServiceDescriptor;

pub struct Config {
    enabled: Vec<ServiceDescriptor>,
    services: BTreeMap<String, ServiceConfig>
}

pub enum ServiceConfig {
    Literal(String),
    File(PathBuf),
}