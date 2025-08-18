use std::{collections::HashMap, fmt::Display, sync::Arc};

use futures::future::BoxFuture;
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    DynRequest, DynResponse,
    flow::{FlowContext, filter::FilterReference},
    instance::{
        InstanceId,
        class::{Class, ClassId, ClassMeta},
    },
};
pub type NodeFn =
    dyn Fn(DynRequest, &mut FlowContext) -> BoxFuture<'_, DynResponse> + Send + Sync + 'static;
#[derive(Clone)]
pub struct Node {
    pub interface: Arc<NodeInterface>,
    pub call: Arc<NodeFn>,
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
    type Config: DeserializeOwned + Serialize + JsonSchema;
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

pub type NodeId = InstanceId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum NodePort {
    Named(Arc<str>),
    #[default]
    Default,
}

impl NodePort {
    pub fn as_str(&self) -> &str {
        match self {
            NodePort::Named(name) => name,
            NodePort::Default => "$default",
        }
    }
}

impl JsonSchema for NodePort {
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> Schema {
        String::json_schema(generator)
    }
    fn schema_name() -> std::borrow::Cow<'static, str> {
        String::schema_name()
    }
    fn inline_schema() -> bool {
        String::inline_schema()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        String::schema_id()
    }
}

impl Serialize for NodePort {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            NodePort::Named(name) => serializer.serialize_str(name),
            NodePort::Default => serializer.serialize_str("$default"),
        }
    }
}

impl<'de> Deserialize<'de> for NodePort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "$default" {
            Ok(NodePort::Default)
        } else {
            Ok(NodePort::Named(Arc::from(s)))
        }
    }
}

impl std::fmt::Display for NodePort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodePort::Named(name) => write!(f, "{}", name),
            NodePort::Default => write!(f, "$default"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NodeInterface {
    pub inputs: HashMap<NodePort, NodeInput>,
    pub outputs: HashMap<NodePort, NodeOutput>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NodeInput {
    pub filters: Vec<FilterReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeOutput {
    pub filters: Vec<FilterReference>,
    pub target: NodeTarget,
}

impl NodeInterface {
    pub fn new(
        inputs: HashMap<NodePort, NodeInput>,
        outputs: HashMap<NodePort, NodeOutput>,
    ) -> Self {
        Self { inputs, outputs }
    }
    pub fn with_default_input(outputs: HashMap<NodePort, NodeOutput>) -> Self {
        Self {
            inputs: HashMap::from_iter([(
                NodePort::Default,
                NodeInput {
                    filters: Vec::new(),
                },
            )]),
            outputs,
        }
    }
    pub fn service() -> Self {
        Self::with_default_input(Default::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct NodeTarget {
    pub id: NodeId,
    pub port: NodePort,
}

impl Display for NodeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.id, self.port)
    }
}

impl NodeTarget {}

pub struct AsNodeClass<N>(pub N);
pub trait NodeClass: Send + Sync + 'static {
    type Node: NodeLike;
    type Error: std::error::Error + Send + Sync + 'static;
    type Config: DeserializeOwned + Serialize + JsonSchema;
    fn id(&self) -> ClassId;
    fn meta(&self) -> ClassMeta {
        ClassMeta::from_env()
    }
    fn schema(&self) -> Schema {
        schema_for!(Self::Config)
    }
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

    fn schema(&self) -> Schema {
        self.0.schema()
    }

    fn construct(
        &self,
        config: Self::Config,
    ) -> Result<crate::instance::InstanceValue, Self::Error> {
        let node = self.0.construct(config)?;
        Ok(crate::instance::InstanceValue::Node(Node::from_node_like(node)))
    }

    fn instance_type(&self) -> crate::instance::InstanceType {
        todo!()
    }
}
