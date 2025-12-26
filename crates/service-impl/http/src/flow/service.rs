pub mod http_client;
pub mod static_response;
pub mod reverse_proxy;
use switchboard_model::services::http::NodeInterface;

use crate::{
    DynRequest, DynResponse,
    flow::{FlowContext, node::NodeLike},
};

pub struct ServiceNode<S> {
    pub service: S,
}

impl<S> ServiceNode<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

pub trait Service: Send + Sync + 'static {
    fn call<'c>(
        &self,
        req: DynRequest,
        ctx: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + Send + 'c;
}

impl<S> NodeLike for ServiceNode<S>
where
    S: Service,
{
    fn call<'c>(
        &self,
        req: DynRequest,
        ctx: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + 'c + Send {
        let req = req;
        self.service.call(req, ctx)
    }

    fn interface(&self) -> NodeInterface {
        NodeInterface::service()
    }
}
