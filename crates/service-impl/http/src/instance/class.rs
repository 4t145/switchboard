pub mod registry;

use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use typeshare::typeshare;

use crate::instance::{InstanceType, InstanceValue};
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, JsonSchema)]
pub struct ClassId {
    pub namespace: Option<String>,
    pub name: String,
}

impl Display for ClassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}.{}", namespace, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl ClassId {
    pub fn std(name: impl Into<String>) -> Self {
        Self {
            namespace: None,
            name: name.into(),
        }
    }
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            name: name.into(),
        }
    }
}
#[typeshare]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ClassMeta {
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
}

impl Default for ClassMeta {
    fn default() -> Self {
        Self::from_env()
    }
}
impl ClassMeta {
    pub fn from_env() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
            author: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            license: Some(env!("CARGO_PKG_LICENSE").to_string()),
            repository: Some(env!("CARGO_PKG_REPOSITORY").to_string()),
            homepage: Some(env!("CARGO_PKG_HOMEPAGE").to_string()),
        }
    }
}
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub struct ClassData {
    pub id: ClassId,
    pub meta: ClassMeta,
    pub instance_type: InstanceType,
    pub config_schema: Schema,
}
pub trait Class: Send + Sync + 'static {
    type Config: DeserializeOwned + Serialize + JsonSchema;
    type Error: std::error::Error + Send + Sync + 'static;
    fn id(&self) -> ClassId;
    fn meta(&self) -> ClassMeta {
        ClassMeta::default()
    }
    fn schema(&self) -> Schema {
        schema_for!(Self::Config)
    }
    fn instance_type(&self) -> InstanceType;
    fn construct(&self, config: Self::Config) -> Result<InstanceValue, Self::Error>;
}

#[derive(Clone)]
pub struct Constructor {
    constructor: Arc<dyn Fn(&serde_json::Value) -> anyhow::Result<InstanceValue> + Send + Sync>,
}

impl Constructor {
    pub fn new<F>(constructor: F) -> Self
    where
        F: Fn(&serde_json::Value) -> anyhow::Result<InstanceValue> + Send + Sync + 'static,
    {
        Self {
            constructor: Arc::new(constructor),
        }
    }
    pub fn construct(&self, name: &serde_json::Value) -> anyhow::Result<InstanceValue> {
        (self.constructor)(name)
    }
}
