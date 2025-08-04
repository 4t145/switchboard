use std::convert::Infallible;

use futures::FutureExt;
use hyper::service::Service;

use crate::{
    DynRequest, DynResponse, DynService,
    flow::{FlowContext, NodeIdentifier, NodeInterface, NodeLike, NodePort},
};

pub struct ServiceNode<S> {
    pub id: NodeIdentifier,
    pub service: S,
}

impl<S> NodeLike for ServiceNode<S>
where
    S: DynService,
{
    fn call<'c>(
        &self,
        req: DynRequest,
        _: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + 'c + Send {
        let req = req;
        let fut = self.service.call(req);
        async move { fut.await.unwrap() }
    }

    fn identifier(&self) -> NodeIdentifier {
        self.id.clone()
    }

    fn interface(&self) -> NodeInterface {
        NodeInterface::service()
    }
}
