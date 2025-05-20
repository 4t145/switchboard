use std::sync::Arc;

use crate::{
    layer::dynamic::SharedLayer,
    router::SharedRouter,
    service::dynamic::{DynService, SharedService},
};

type Make<T> = dyn Fn(&str) -> anyhow::Result<T>;

#[derive(Clone)]
pub struct ServiceMaker {
    make: Arc<Make<SharedService>>,
}

impl ServiceMaker {
    pub fn new<F, E>(make: F) -> Self
    where
        F: Fn(&str) -> Result<SharedService, E> + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            make: Arc::new(move |name| make(name).map_err(|e| anyhow::anyhow!(e))),
        }
    }
    pub fn make(&self, name: &str) -> anyhow::Result<SharedService> {
        (self.make)(name)
    }
}
#[derive(Clone)]
pub struct LayerMaker {
    make: Arc<Make<SharedLayer>>,
}

impl LayerMaker {
    pub fn new<F, E>(make: F) -> Self
    where
        F: Fn(&str) -> Result<SharedLayer, E> + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            make: Arc::new(move |name| make(name).map_err(|e| anyhow::anyhow!(e))),
        }
    }
    pub fn make(&self, name: &str) -> anyhow::Result<SharedLayer> {
        (self.make)(name)
    }
}
#[derive(Clone)]
pub struct RouterMaker {
    make: Arc<Make<SharedRouter>>,
}

impl RouterMaker {
    pub fn new<F, E>(make: F) -> Self
    where
        F: Fn(&str) -> Result<SharedRouter, E> + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            make: Arc::new(move |name| make(name).map_err(|e| anyhow::anyhow!(e))),
        }
    }
    pub fn make(&self, name: &str) -> anyhow::Result<SharedRouter> {
        (self.make)(name)
    }
}

pub enum NodeMaker {
    Router(RouterMaker),
    Layer(LayerMaker),
    Service(ServiceMaker),
}