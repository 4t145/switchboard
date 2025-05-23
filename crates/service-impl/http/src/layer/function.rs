use std::sync::Arc;

use crate::service::{
    dynamic::{DynRequest, DynResponse, DynService, SharedService},
    function::FunctionService,
};

use super::dynamic::{DynLayer, SharedLayer};
#[derive(Clone)]
pub struct Inner {
    inner: SharedService,
}

impl Inner {
    pub fn new(inner: SharedService) -> Self {
        Self { inner }
    }
    pub async fn call(&self, req: DynRequest) -> DynResponse {
        DynService::call(&self.inner, req)
            .await
            .expect("infallible")
    }
}

#[derive(Clone)]
pub struct FunctionLayer<M: LayerMethod> {
    method: Arc<M>,
}

impl<M: LayerMethod> FunctionLayer<M> {
    pub fn new(method: M) -> Self {
        Self {
            method: Arc::new(method),
        }
    }
}

impl SharedLayer {
    pub fn function<L>(layer: L) -> Self
    where
        L: LayerMethod + 'static,
    {
        Self::new(FunctionLayer::new(layer))
    }
}

pub trait LayerMethod: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    fn error_kind() -> &'static str {
        std::any::type_name::<Self>()
    }
    fn call(
        self: Arc<Self>,
        req: DynRequest,
        inner: Inner,
    ) -> impl Future<Output = Result<DynResponse, Self::Error>> + Send + 'static;
}

impl<F> DynLayer for FunctionLayer<F>
where
    F: LayerMethod,
{
    fn layer(&self, service: SharedService) -> SharedService {
        let inner = Inner::new(service);
        let function = self.method.clone();
        let error_kind = F::error_kind();
        let function = move |req: DynRequest| {
            let inner = inner.clone();
            let function = function.clone();
            Box::pin(function.call(req, inner))
        };
        let service = FunctionService::new(function, error_kind);
        let service = SharedService::new(service);
        service
    }
}
