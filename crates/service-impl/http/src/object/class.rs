pub mod layer;
pub mod router;
pub mod service;

use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct ObjectClassName {
    pub namespace: Option<String>,
    pub name: String,
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
    pub document: Option<String>,
    pub schema: Option<schemars::schema::RootSchema>,
    pub kind: ObjectClassKindEnum,
}

pub struct ObjectClass<C: ObjectClassType> {
    pub name: ObjectClassName,
    pub meta: ObjectClassMeta,
    pub constructor: Constructor<C>,
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
