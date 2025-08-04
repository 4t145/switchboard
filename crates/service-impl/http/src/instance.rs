pub mod class;
// pub mod orchestration;
// pub mod registry;

use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    fmt::Display,
    ops::Deref,
    str::FromStr,
    sync::Arc,
};

use class::ClassId;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct InstanceId(Arc<str>);

impl Deref for InstanceId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl std::fmt::Debug for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjectId({})", self.0)
    }
}

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

impl InstanceId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
    pub fn random() -> Self {
        Self(Arc::from(uuid::Uuid::new_v4().to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortKey {
    Named(String),
    Default,
}

impl PortKey {
    pub fn as_str(&self) -> &str {
        match self {
            PortKey::Named(name) => name,
            PortKey::Default => "$default",
        }
    }
}

impl Display for PortKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortKey::Named(name) => write!(f, "{}", name),
            PortKey::Default => write!(f, "$default"),
        }
    }
}

impl FromStr for PortKey {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "$default" {
            Ok(PortKey::Default)
        } else {
            Ok(PortKey::Named(s.to_string()))
        }
    }
}

impl Serialize for PortKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PortKey::Named(name) => serializer.serialize_str(name),
            PortKey::Default => serializer.serialize_str("$default"),
        }
    }
}

impl<'de> Deserialize<'de> for PortKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PortKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub instance: InstanceId,
    pub port: PortKey,
}

impl Target {
    pub fn default_of(instance: InstanceId) -> Self {
        Self {
            instance,
            port: PortKey::Default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    input: HashSet<PortKey>,
    output: HashMap<PortKey, Target>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ClassKind {
    Bundle,
    Router,
    Layer,
    Service,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Instance {
    pub id: InstanceId,
    pub class: ClassId,
    pub interface: Interface,
    pub kind: ClassKind,
    pub config: serde_json::Value,
}
