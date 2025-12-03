use std::{convert::Infallible, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, Hash, bon::Builder, bincode::Encode, bincode::Decode, PartialEq, Eq)]
#[builder(on(String, into))]
pub struct AnonServiceDescriptor {
    pub provider: String,
    pub tls: Option<String>,
    pub config: Option<String>,
}

pub type NamedServiceDescriptor = String;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub enum ServiceDescriptor {
    Anon(AnonServiceDescriptor),
    Named(NamedServiceDescriptor),
}

impl From<AnonServiceDescriptor> for ServiceDescriptor {
    fn from(value: AnonServiceDescriptor) -> Self {
        ServiceDescriptor::Anon(value)
    }
}

impl From<NamedServiceDescriptor> for ServiceDescriptor {
    fn from(value: NamedServiceDescriptor) -> Self {
        ServiceDescriptor::Named(value)
    }
}

impl ServiceDescriptor {
    pub fn named(name: impl Into<String>) -> Self {
        ServiceDescriptor::Named(name.into())
    }
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
                write!(f, "{}", anon.provider)?;
                if let Some(tls) = &anon.tls {
                    write!(f, ":{}", tls)?;
                }
                if let Some(config) = &anon.config {
                    write!(f, "/{}", config)?;
                }
                Ok(())
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
        } else {
            let mut config = None;
            let mut tls = None;
            let provider_and_tls = if let Some((service, config_str)) = s.split_once('/') {
                config = Some(config_str.to_string());
                service
            } else {
                s
            };
            let provider = if let Some((provider_str, tls_str)) = provider_and_tls.split_once(':') {
                tls = Some(tls_str.to_string());
                provider_str
            } else {
                provider_and_tls
            };
            Ok(ServiceDescriptor::Anon(AnonServiceDescriptor {
                provider: provider.to_owned(),
                tls,
                config,
            }))
        }
    }
}

#[test]
fn test_descriptor() {
    let raw_str = "pf/111.222.111.222:1122";
    let x = raw_str.parse::<ServiceDescriptor>().expect("fail to parse");
    assert!(x.is_anon());
    assert_eq!(x.to_string(), raw_str.to_owned());

    let raw_str = "@pf.remote";
    let x = raw_str.parse::<ServiceDescriptor>().expect("fail to parse");
    assert!(x.is_named());
    assert_eq!(x.to_string(), raw_str.to_owned());

    let raw_str = "pf:4t145/111.222.111.222:1122";
    let x = raw_str.parse::<ServiceDescriptor>().expect("fail to parse");
    matches!(
        &x,
        ServiceDescriptor::Anon(AnonServiceDescriptor {
            provider: _,
            tls: Some(_),
            config: Some(_)
        })
    );
    assert_eq!(x.to_string(), raw_str.to_owned());
}
