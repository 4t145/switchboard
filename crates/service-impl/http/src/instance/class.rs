pub mod registry;

use std::sync::Arc;

use serde::de::DeserializeOwned;
use switchboard_model::services::http::*;
use switchboard_service::SerdeValue;

use crate::instance::InstanceValue;

pub trait Class: Send + Sync + 'static {
    type Config: DeserializeOwned;
    type Error: std::error::Error + Send + Sync + 'static;
    fn id(&self) -> ClassId;
    fn meta(&self) -> ClassMeta {
        ClassMeta::default()
    }
    fn instance_type(&self) -> InstanceType;
    fn construct(&self, config: Self::Config) -> Result<InstanceValue, Self::Error>;
}

type ConstructorFn = dyn Fn(&SerdeValue) -> Result<InstanceValue, ConstructError> + Send + Sync;
#[derive(Clone)]
pub struct Constructor {
    constructor: Arc<ConstructorFn>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConstructError {
    #[error("Config decode error: {0}")]
    ConfigDecodeError(#[from] switchboard_model::switchboard_serde_value::Error),
    #[error("Build error: {0}")]
    BuildError(Box<dyn std::error::Error + Send + Sync>),
}
impl Constructor {
    pub fn from_class<C: Class>(class: C) -> Self {
        Self {
            constructor: Arc::new(move |config: &SerdeValue| {
                let cfg: C::Config = config
                    .clone()
                    .deserialize_into()
                    .map_err(ConstructError::ConfigDecodeError)?;
                class
                    .construct(cfg)
                    .map_err(|e| ConstructError::BuildError(Box::new(e)))
            }),
        }
    }
    pub fn new<F>(constructor: F) -> Self
    where
        F: Fn(&SerdeValue) -> Result<InstanceValue, ConstructError> + Send + Sync + 'static,
    {
        Self {
            constructor: Arc::new(constructor),
        }
    }
    pub fn construct(&self, config: &SerdeValue) -> Result<InstanceValue, ConstructError> {
        (self.constructor)(config)
    }
}
