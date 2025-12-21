use std::collections::BTreeMap;
// pub mod host_match;
// pub mod path_match;
pub mod balancer;
#[allow(clippy::module_inception)]
pub mod router;
pub mod transparent;

use crate::{
    DynRequest, DynResponse,
    flow::{FlowContext, node::NodeLike},
};

use switchboard_model::services::http::{NodeInterface, NodeOutput, NodePort};

pub trait Router: Send + Sync + 'static {
    fn route(&self, req: &mut http::request::Parts) -> NodePort;
}

pub struct RouterNode<R: Router> {
    pub routes: BTreeMap<NodePort, NodeOutput>,
    pub router: R,
}

impl<R: Router> RouterNode<R> {
    pub fn new(routes: BTreeMap<NodePort, NodeOutput>, router: R) -> Self {
        Self { routes, router }
    }
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

    fn interface(&self) -> NodeInterface {
        NodeInterface::with_default_input(self.routes.clone())
    }
}
