use std::collections::HashMap;

use crate::{
    DynRequest, DynResponse,
    flow::{FlowContext, NodeIdentifier, NodeInterface, NodeLike, NodePort, NodeTarget},
};

pub struct Next<'c> {
    pub context: &'c mut FlowContext,
}

impl<'c> Next<'c> {
    pub(crate) fn new(context: &'c mut FlowContext) -> Self {
        Self { context }
    }

    pub fn call(self, req: DynRequest) -> impl futures::Future<Output = DynResponse> + 'c + Send {
        let port = self.context.current_state.input_port.clone();
        let context = self.context;
        async move {
            let response = context.call(req, port).await;
            response
        }
    }
}

pub trait Plane {
    fn call<'c>(
        &self,
        req: DynRequest,
        context: Next<'c>,
    ) -> impl futures::Future<Output = DynResponse> + 'c + Send;
}

pub struct PlaneNode<P: Plane> {
    pub id: NodeIdentifier,
    pub ports: HashMap<NodePort, NodeTarget>,
    pub plane: P,
}

impl<P: Plane> NodeLike for PlaneNode<P> {
    fn call<'c>(
        &self,
        req: DynRequest,
        context: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + 'c + Send {
        let next = Next::new(context);
        self.plane.call(req, next)
    }

    fn identifier(&self) -> NodeIdentifier {
        self.id.clone()
    }

    fn interface(&self) -> NodeInterface {
        NodeInterface::new(self.ports.keys().cloned().collect(), self.ports.clone())
    }
}
