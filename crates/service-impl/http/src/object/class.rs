mod layer;
pub use layer::*;
mod router;
pub use router::*;
mod service;
pub use service::*;

use std::{fmt::{Debug, Display}, sync::Arc};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ObjectClassName {
    pub namespace: Option<String>,
    pub name: String,
}

impl Display for ObjectClassName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}.{}", namespace, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl ObjectClassName {
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ObjectClassKindEnum {
    Layer,
    Service,
    Router,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectClassMeta {
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub schema: Option<schemars::schema::RootSchema>,
}
impl Default for ObjectClassMeta {
    fn default() -> Self {
        Self::from_env()
    }
}
impl ObjectClassMeta {
    pub fn from_env() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
            author: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            license: Some(env!("CARGO_PKG_LICENSE").to_string()),
            repository: Some(env!("CARGO_PKG_REPOSITORY").to_string()),
            homepage: Some(env!("CARGO_PKG_HOMEPAGE").to_string()),
            schema: None,
        }
    }
}

pub struct ObjectClass<C: ObjectClassType> {
    pub name: ObjectClassName,
    pub meta: ObjectClassMeta,
    pub constructor: Constructor<C>,
}

impl<C: ObjectClassType> ObjectClass<C> {
    pub fn from_sbh_class<Class>(class: Class) -> Self
    where
        Class: SbhClass<Type = C>,
    {
        Self {
            name: class.name(),
            meta: class.meta(),
            constructor: Constructor::new(move |config| class.construct(config)),
        }
    }
}

pub trait SbhClass: Send + Sync + 'static {
    type Type: ObjectClassType;
    type Error: std::error::Error + Send + Sync + 'static;
    fn name(&self) -> ObjectClassName;
    fn meta(&self) -> ObjectClassMeta {
        ObjectClassMeta::default()
    }
    fn construct(&self, config: &str) -> Result<Self::Type, Self::Error>;
}

impl<C: ObjectClassType> Clone for ObjectClass<C> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            meta: self.meta.clone(),
            constructor: self.constructor.clone(),
        }
    }
}

impl<C: ObjectClassType> std::fmt::Debug for ObjectClass<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectClass")
            .field("name", &self.name)
            .field("meta", &self.meta)
            .finish()
    }
}

pub trait ObjectClassType {
    type Property: Serialize + DeserializeOwned + Clone + Debug;
    const KIND: ObjectClassKindEnum;
}

pub struct Constructor<C: ObjectClassType> {
    constructor: Arc<dyn Fn(&str) -> anyhow::Result<C> + Send + Sync>,
}

impl<C: ObjectClassType> Clone for Constructor<C> {
    fn clone(&self) -> Self {
        Self {
            constructor: self.constructor.clone(),
        }
    }
}

impl<C: ObjectClassType> Constructor<C> {
    pub fn new<F, E>(constructor: F) -> Self
    where
        F: Fn(&str) -> Result<C, E> + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            constructor: Arc::new(move |name| constructor(name).map_err(|e| anyhow::anyhow!(e))),
        }
    }
    pub fn construct(&self, name: &str) -> anyhow::Result<C> {
        (self.constructor)(name)
    }
}
