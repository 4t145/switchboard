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
    constructor: Arc<dyn Fn(&CustomConfig) -> anyhow::Result<InstanceValue> + Send + Sync>,
}

impl Constructor {
    pub fn new<F>(constructor: F) -> Self
    where
        F: Fn(&CustomConfig) -> anyhow::Result<InstanceValue> + Send + Sync + 'static,
    {
        Self {
            constructor: Arc::new(constructor),
        }
    }
    pub fn construct(&self, name: &CustomConfig) -> anyhow::Result<InstanceValue> {
        (self.constructor)(name)
    }
}
