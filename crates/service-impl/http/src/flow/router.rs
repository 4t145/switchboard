use std::collections::HashMap;
pub mod host_match;
pub mod path_match;
pub mod transparent;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    DynRequest, DynResponse,
    flow::{
        FlowContext, NodePort,
        node::{NodeInterface, NodeLike, NodeOutput},
    },
};
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub struct WithRoutes<C> {
    pub router_config: C,
    pub routes: HashMap<NodePort, NodeOutput>,
}
pub trait Router: Send + Sync + 'static {
    fn route(&self, req: &mut http::request::Parts) -> NodePort;
}

pub struct RouterNode<R: Router> {
    pub routes: HashMap<NodePort, NodeOutput>,
    pub router: R,
}

impl<R: Router> RouterNode<R> {
    pub fn new(routes: HashMap<NodePort, NodeOutput>, router: R) -> Self {
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
