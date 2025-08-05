mod client;
use crate::{
    DynRequest, DynResponse,
    flow::{
        FlowContext,
        node::{NodeIdentifier, NodeInterface, NodeLike},
    },
};

pub struct ServiceNode<S> {
    pub id: NodeIdentifier,
    pub service: S,
}

pub trait Service {
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
        let fut = self.service.call(req, ctx);
        fut
    }

    fn identifier(&self) -> NodeIdentifier {
        self.id.clone()
    }

    fn interface(&self) -> NodeInterface {
        NodeInterface::service()
    }
}
