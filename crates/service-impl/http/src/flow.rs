use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::Arc,
};
mod router;
mod service;
mod plane;
mod filter;
use bytes::Bytes;
use futures::future::BoxFuture;
use http::{Request, StatusCode};
use hyper::body::Body;
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    DynRequest, DynResponse, IntoDynResponse, box_error,
    instance::class::{ClassId, ClassMeta},
    utils::error_response,
};
#[derive(Clone)]
pub struct Node {
    pub identifier: Arc<NodeIdentifier>,
    pub interface: Arc<NodeInterface>,
    pub handler: Arc<Handler>,
}

impl Node {
    pub fn new<F>(identifier: NodeIdentifier, interface: NodeInterface, handler: F) -> Self
    where
        F: Fn(DynRequest, &mut FlowContext) -> BoxFuture<'_, DynResponse> + Send + Sync + 'static,
    {
        Self {
            identifier: Arc::new(identifier),
            interface: Arc::new(interface),
            handler: Arc::new(handler),
        }
    }
    pub fn from_node_like<N>(node: N) -> Self
    where
        N: NodeLike + Send + Sync + 'static,
    {
        Self::new(node.identifier(), node.interface(), move |req, context| {
            Box::pin(node.call(req, context))
        })
    }
}

pub trait NodeLike {
    fn call<'c>(
        &self,
        req: DynRequest,
        context: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + 'c + Send;
    fn identifier(&self) -> NodeIdentifier;
    fn interface(&self) -> NodeInterface;
}
pub trait IntoNode {
    fn into_node(self) -> Node;
}

pub trait NodeClass {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(Arc<str>);
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum NodePort {
    Named(Arc<str>),
    #[default]
    Default,
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

#[derive(Clone)]
pub struct NodeIdentifier {
    pub id: NodeId,
    pub name: Option<String>,
}

impl NodeIdentifier {
    pub fn new(id: impl Into<Arc<str>>, name: Option<String>) -> Self {
        Self {
            id: NodeId(id.into()),
            name,
        }
    }
}

#[derive(Clone)]
pub struct NodeInterface {
    inputs: HashSet<NodePort>,
    outputs: HashMap<NodePort, NodeTarget>,
}

impl NodeInterface {
    pub fn new(inputs: HashSet<NodePort>, outputs: HashMap<NodePort, NodeTarget>) -> Self {
        Self { inputs, outputs }
    }
    pub fn with_default_input(outputs: HashMap<NodePort, NodeTarget>) -> Self {
        Self {
            inputs: HashSet::from_iter([NodePort::Default]),
            outputs,
        }
    }
    pub fn service() -> Self {
        Self::with_default_input(Default::default())
    }
    pub fn incoming(next: NodeTarget) -> Self {
        Self {
            inputs: HashSet::new(),
            outputs: HashMap::from_iter(Some((NodePort::Default, next))),
        }
    }
}

pub struct NodeInputs(HashSet<NodeId>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeTarget {
    pub id: NodeId,
    pub port: NodePort,
}

impl NodeTarget {}

pub struct NodeOutputs(HashMap<NodePort, NodeTarget>);
#[derive(Clone)]
pub struct Flow {
    pub nodes: Arc<HashMap<NodeId, Node>>,
    pub entrypoint: NodeId,
}

impl Flow {}

#[derive(Default, Debug, Clone)]
pub struct FlowTrace {
    pub finished: Vec<FlowContextState>,
    pub pending: Vec<FlowContextState>,
}

impl FlowTrace {
    pub fn loop_count_at(&self, node: &NodeId) -> usize {
        self.pending
            .iter()
            .filter(|state| &state.node == node)
            .count()
    }
}

pub struct FlowContext {
    pub flow: Flow,
    pub current_state: FlowContextState,
    pub trace: FlowTrace,
    pub config: FlowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowConfig {
    pub max_loop: Option<u32>,
}
#[derive(Debug, Clone)]
pub struct FlowContextState {
    pub node: NodeId,
    pub input_port: NodePort,
}

#[derive(Debug, thiserror::Error)]
pub enum FlowError {
    #[error("Node `{0}` not found in the flow")]
    NodeNotFound(NodeId),
    #[error("Invalid port `{0}` for the current node")]
    InvalidPort(NodePort),
    #[error("Loop detected at node {node}, out of limit {limit}, trace: {trace:#?}")]
    LoopDetected {
        node: NodeId,
        limit: u32,
        trace: FlowTrace,
    },
}

impl IntoDynResponse for FlowError {
    fn into_dyn_response(self) -> DynResponse {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, self, "FLOW_ERROR")
    }
}

impl FlowContext {
    pub fn new(flow: Flow, entrypoint: NodeId) -> Self {
        Self {
            flow,
            current_state: FlowContextState {
                node: entrypoint,
                input_port: NodePort::Default,
            },
            trace: FlowTrace::default(),
            config: FlowConfig::default(),
        }
    }

    pub fn set_state(&mut self, state: FlowContextState) {
        self.current_state = state;
    }

    pub fn get_entry_node(&self) -> Result<&Node, FlowError> {
        self.flow
            .nodes
            .get(&self.flow.entrypoint)
            .ok_or_else(|| FlowError::NodeNotFound(self.flow.entrypoint.clone()))
    }
    pub fn get_current_node(&self) -> Result<&Node, FlowError> {
        self.flow
            .nodes
            .get(&self.current_state.node)
            .ok_or_else(|| FlowError::NodeNotFound(self.current_state.node.clone()))
    }
    pub fn entry(&mut self, state: FlowContextState) {
        let old_state = std::mem::replace(&mut self.current_state, state);
        self.trace.pending.push(old_state);
    }

    pub fn entry_target(&mut self, target: NodeTarget) {
        let state = FlowContextState {
            node: target.id,
            input_port: target.port,
        };
        self.entry(state);
    }

    pub fn leave(&mut self) {
        if let Some(node) = self.trace.pending.pop() {
            self.trace.finished.push(node);
        }
        if let Some(current) = self.trace.pending.pop() {
            self.current_state = current;
        } else {
            self.current_state = FlowContextState {
                node: self.flow.entrypoint.clone(),
                input_port: NodePort::Default,
            };
        }
    }

    async fn call_next_with_error(
        &mut self,
        req: DynRequest,
        port: NodePort,
    ) -> Result<DynResponse, FlowError> {
        let node = self.get_current_node()?;
        let target = node
            .interface
            .outputs
            .get(&port)
            .ok_or_else(|| FlowError::InvalidPort(port))?
            .clone();
        if let Some(max_loop) = self.config.max_loop {
            if self.trace.loop_count_at(&target.id) > max_loop as usize {
                return Err(FlowError::LoopDetected {
                    node: target.id.clone(),
                    limit: max_loop,
                    trace: self.trace.clone(),
                });
            }
        }
        let next_node = self
            .flow
            .nodes
            .get(&target.id)
            .ok_or_else(|| FlowError::NodeNotFound(target.id.clone()))?
            .clone();
        self.entry_target(target);
        let response = (next_node.handler)(req, self).await;
        Ok(response)
    }

    pub async fn call(&mut self, req: DynRequest, port: NodePort) -> DynResponse {
        match self.call_next_with_error(req, port).await {
            Ok(response) => {
                self.leave();
                response
            }
            Err(e) => {
                tracing::error!("Flow error: {}", e);
                self.leave();
                e.into_dyn_response()
            }
        }
    }

    pub async fn call_default(&mut self, req: DynRequest) -> DynResponse {
        self.call(req, NodePort::Default).await
    }

    pub fn trace(&self) -> &FlowTrace {
        &self.trace
    }
}

pub type Handler =
    dyn Fn(DynRequest, &mut FlowContext) -> BoxFuture<'_, DynResponse> + Send + Sync + 'static;

impl<Req> hyper::service::Service<Request<Req>> for Flow
where
    Req: Body<Data = Bytes> + Send + 'static,
    <Req as Body>::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = DynResponse;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Req>) -> Self::Future {
        let flow = self.clone();
        let req = req.map(|body| {
            use http_body_util::BodyExt;
            body.map_err(box_error).boxed_unsync()
        });
        Box::pin(async move {
            let entrypoint = flow.entrypoint.clone();
            let mut context = FlowContext::new(flow, entrypoint);
            let entry_node = match context.get_entry_node() {
                Ok(node) => node,
                Err(e) => {
                    tracing::error!("Failed to get entry node: {}", e);
                    return Ok(e.into_dyn_response());
                }
            };
            let response = (entry_node.handler.clone())(req, &mut context).await;
            Ok(response)
        })
    }
}
