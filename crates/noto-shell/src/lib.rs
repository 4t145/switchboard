use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
    fmt::Display,
    net::SocketAddr,
    str::FromStr,
};

pub struct AnonServiceDescriptor {
    service: String,
    config_str: Option<String>,
}

pub type NamedServiceDescriptor = String;
pub enum ServiceDescriptor {
    Anon(AnonServiceDescriptor),
    Named(NamedServiceDescriptor),
}
impl Display for ServiceDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceDescriptor::Anon(anon) => {
                if let Some(config_str) = &anon.config_str {
                    write!(f, "{}/{}", anon.service, config_str)
                } else {
                    write!(f, "{}", anon.service)
                }
            }
            ServiceDescriptor::Named(named) => write!(f, "@{}", named),
        }
    }
}
impl FromStr for ServiceDescriptor {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(resource_name) = s.strip_prefix('@') {
            Ok(ServiceDescriptor::Named(resource_name.to_owned()))
        } else if let Some((service, config_str)) = s.split_once('/') {
            Ok(ServiceDescriptor::Anon(AnonServiceDescriptor {
                service: service.to_owned(),
                config_str: Some(config_str.to_owned()),
            }))
        } else {
            Ok(ServiceDescriptor::Anon(AnonServiceDescriptor {
                service: s.to_owned(),
                config_str: None,
            }))
        }
    }
}

pub struct NotoItem {
    pub bind: SocketAddr,
    pub service: ServiceDescriptor,
    pub description: Option<String>,
    pub tags: BTreeSet<String>,
}

pub struct NotoConfig {
    pub items: Vec<NotoItem>,
}