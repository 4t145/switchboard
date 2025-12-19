pub mod registry;

use std::sync::Arc;

use switchboard_model::services::http::*;
use switchboard_service::CustomConfig;

use crate::instance::InstanceValue;

pub trait Class: Send + Sync + 'static {
    type Config: switchboard_service::PayloadObject;
    type Error: std::error::Error + Send + Sync + 'static;
    fn id(&self) -> ClassId;
    fn meta(&self) -> ClassMeta {
        ClassMeta::default()
    }
    fn instance_type(&self) -> InstanceType;
    fn construct(&self, config: Self::Config) -> Result<InstanceValue, Self::Error>;
}

#[derive(Clone)]
pub struct Constructor {
    constructor: Arc<dyn Fn(&CustomConfig) -> Result<InstanceValue, ConstructError> + Send + Sync>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConstructError {
    #[error("Config decode error: {0}")]
    ConfigDecodeError(switchboard_model::custom_config::Error),
    #[error("Build error: {0}")]
    BuildError(Box<dyn std::error::Error + Send + Sync>),
}
impl Constructor {
    pub fn from_class<C: Class>(class: C) -> Self {
        Self {
            constructor: Arc::new(move |config: &CustomConfig| {
                let cfg: C::Config = config
                    .clone().decode()
                    .map_err(ConstructError::ConfigDecodeError)?;
                class
                    .construct(cfg)
                    .map_err(|e| ConstructError::BuildError(Box::new(e)))
            }),
        }
    }
    pub fn new<F>(constructor: F) -> Self
    where
        F: Fn(&CustomConfig) -> Result<InstanceValue, ConstructError> + Send + Sync + 'static,
    {
        Self {
            constructor: Arc::new(constructor),
        }
    }
    pub fn construct(&self, config: &CustomConfig) -> Result<InstanceValue, ConstructError> {
        (self.constructor)(config)
    }
}
