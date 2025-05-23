use std::sync::Arc;

use crate::service::dynamic::{DynService, SharedService};

use super::Layer;

pub trait DynLayer {
    fn layer(&self, service: SharedService) -> SharedService;
}
#[derive(Clone)]
pub struct SharedLayer {
    layer: Arc<dyn DynLayer>,
}

impl SharedLayer {
    pub fn new<L>(layer: L) -> Self
    where
        L: DynLayer + 'static,
    {
        Self {
            layer: Arc::new(layer),
        }
    }
}

impl<L> DynLayer for L
where
    L: Layer<SharedService> + Clone,
    L::Service: DynService,
{
    fn layer(&self, service: SharedService) -> SharedService {
        SharedService::new(self.clone().layer(service))
    }
}

impl Layer<SharedService> for SharedLayer {
    type Service = SharedService;

    fn layer(self, service: SharedService) -> Self::Service {
        self.layer.layer(service)
    }
}
