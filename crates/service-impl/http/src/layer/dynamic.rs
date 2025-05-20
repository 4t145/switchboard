use std::sync::Arc;

use crate::service::dynamic::{DynService, SharedService};

use super::Layer;

pub trait DynLayer {
    fn layer(&self, service: SharedService) -> SharedService;
}

pub struct SharedLayer {
    layer: Arc<dyn DynLayer>,
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
