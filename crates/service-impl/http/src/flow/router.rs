use std::collections::HashMap;

use crate::{
    DynRequest, DynResponse,
    flow::{FlowContext, NodeIdentifier, NodeInterface, NodeLike, NodePort, NodeTarget},
};

pub trait Router: Send + Sync + 'static {
    fn route(&self, req: &mut http::request::Parts) -> NodePort;
}

pub struct RouterNode<R: Router> {
    pub id: NodeIdentifier,
    pub routes: HashMap<NodePort, NodeTarget>,
    pub router: R,
}

impl<R: Router> NodeLike for RouterNode<R> {
    fn call<'c>(
        &self,
        req: DynRequest,
        context: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + 'c + Send {
        let req = req;
        let (mut parts, body) = req.into_parts();
        let port = self.router.route(&mut parts);
        let req = DynRequest::from_parts(parts, body);
        context.call(req, port)
    }

    fn identifier(&self) -> NodeIdentifier {
        self.id.clone()
    }

    fn interface(&self) -> NodeInterface {
        NodeInterface::with_default_input(self.routes.clone())
    }
}
