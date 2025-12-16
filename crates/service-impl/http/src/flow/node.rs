use std::sync::Arc;

use futures::future::BoxFuture;
use schemars::{JsonSchema, Schema, schema_for};

use crate::{DynRequest, DynResponse, flow::FlowContext, instance::class::Class};

use switchboard_model::services::http::{ClassId, ClassMeta, InstanceType, NodeInterface};
pub type NodeFn =
    dyn Fn(DynRequest, &mut FlowContext) -> BoxFuture<'_, DynResponse> + Send + Sync + 'static;
#[derive(Clone)]
pub struct Node {
    pub interface: Arc<NodeInterface>,
    pub call: Arc<NodeFn>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("interface", &self.interface)
            .finish()
    }
}

impl Node {
    pub fn new<F>(interface: NodeInterface, handler: F) -> Self
    where
        F: Fn(DynRequest, &mut FlowContext) -> BoxFuture<'_, DynResponse> + Send + Sync + 'static,
    {
        Self {
            interface: Arc::new(interface),
            call: Arc::new(handler),
        }
    }
    pub fn from_node_like<N>(node: N) -> Self
    where
        N: NodeLike + Send + Sync + 'static,
    {
        Self::new(node.interface(), move |req, context| {
            Box::pin(node.call(req, context))
        })
    }
}

pub trait NodeLike: Send + Sync + 'static {
    fn call<'c>(
        &self,
        req: DynRequest,
        context: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + 'c + Send;

    fn interface(&self) -> NodeInterface;
}
pub trait IntoNode {
    fn into_node(self) -> Node;
}

pub trait NodeType {
    type Config: switchboard_service::PayloadObject + JsonSchema;
    type Error: std::error::Error + Send + Sync + 'static;
    fn id(&self) -> ClassId;
    fn meta(&self) -> ClassMeta {
        ClassMeta::default()
    }
    fn schema(&self) -> Schema {
        schema_for!(Self::Config)
    }
    fn construct(&self, config: Self::Config) -> Result<Node, Self::Error>;
}

pub struct AsNodeClass<N>(pub N);
pub trait NodeClass: Send + Sync + 'static {
    type Node: NodeLike;
    type Error: std::error::Error + Send + Sync + 'static;
    type Config: switchboard_service::PayloadObject;
    fn id(&self) -> ClassId;
    fn meta(&self) -> ClassMeta {
        ClassMeta::from_env()
    }
    // fn schema(&self) -> Schema {
    //     schema_for!(Self::Config)
    // }
    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error>;
}

impl<N> Class for AsNodeClass<N>
where
    N: NodeClass,
{
    type Config = <N as NodeClass>::Config;

    type Error = <N as NodeClass>::Error;
    fn id(&self) -> ClassId {
        self.0.id()
    }

    fn meta(&self) -> ClassMeta {
        self.0.meta()
    }

    fn construct(
        &self,
        config: Self::Config,
    ) -> Result<crate::instance::InstanceValue, Self::Error> {
        let node = self.0.construct(config)?;
        Ok(crate::instance::InstanceValue::Node(Node::from_node_like(
            node,
        )))
    }

    fn instance_type(&self) -> InstanceType {
        InstanceType::Node
    }
}
