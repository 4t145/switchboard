use std::{convert::Infallible, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(bon::Builder)]
#[builder(on(String, into))]
pub struct AnonServiceDescriptor {
    pub provider: String,
    pub config: Option<String>,
}

pub type NamedServiceDescriptor = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServiceDescriptor {
    Anon(AnonServiceDescriptor),
    Named(NamedServiceDescriptor),
}

impl ServiceDescriptor {
    pub fn is_anon(&self) -> bool {
        matches!(self, ServiceDescriptor::Anon(_))
    }
    pub fn is_named(&self) -> bool {
        matches!(self, ServiceDescriptor::Named(_))
    }
}

impl Display for ServiceDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceDescriptor::Anon(anon) => {
                if let Some(config_str) = &anon.config {
                    write!(f, "{}/{}", anon.provider, config_str)
                } else {
                    write!(f, "{}", anon.provider)
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
                provider: service.to_owned(),
                config: Some(config_str.to_owned()),
            }))
        } else {
            Ok(ServiceDescriptor::Anon(AnonServiceDescriptor {
                provider: s.to_owned(),
                config: None,
            }))
        }
    }
}

#[test]
fn test_descriptor() {
    let raw_str = "tf/111.222.111.222:1122";
    let x = raw_str.parse::<ServiceDescriptor>().expect("fail to parse");
    assert!(x.is_anon());
    assert_eq!(x.to_string(), raw_str.to_owned());
    let raw_str = "@tf.remote";

    let x = raw_str.parse::<ServiceDescriptor>().expect("fail to parse");
    assert!(x.is_named());
    assert_eq!(x.to_string(), raw_str.to_owned());
}
