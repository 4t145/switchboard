use std::{collections::HashMap, convert::Infallible, sync::Arc};
pub mod filter;
pub mod node;
pub mod router;
pub mod service;
use bytes::Bytes;
use futures::future::BoxFuture;
use http::{Request, StatusCode};
use hyper::body::Body;
use serde::{Deserialize, Serialize};

use crate::{
    box_error, flow::node::{Node, NodeId, NodePort, NodeTarget}, utils::error_response, DynRequest, DynResponse, IntoDynResponse, ERR_FLOW
};

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
        error_response(StatusCode::INTERNAL_SERVER_ERROR, self, ERR_FLOW)
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

    pub fn entry(&mut self, target: NodeTarget) {
        let state = FlowContextState {
            node: target.id,
            input_port: target.port,
        };
        let old_state = std::mem::replace(&mut self.current_state, state);
        self.trace.pending.push(old_state);
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
        let output = node
            .interface
            .outputs
            .get(&port)
            .ok_or_else(|| FlowError::InvalidPort(port))?
            .clone();
        if let Some(max_loop) = self.config.max_loop {
            if self.trace.loop_count_at(&output.target.id) > max_loop as usize {
                return Err(FlowError::LoopDetected {
                    node: output.target.id.clone(),
                    limit: max_loop,
                    trace: self.trace.clone(),
                });
            }
        }
        let target = output.target;
        let output_filters = output.filters.clone();
        let next_node = self
            .flow
            .nodes
            .get(&target.id)
            .ok_or_else(|| FlowError::NodeNotFound(target.id.clone()))?
            .clone();
        let input_filters = next_node
            .interface
            .inputs
            .get(&target.port)
            .ok_or_else(|| FlowError::InvalidPort(target.port.clone()))?
            .filters
            .clone();
        let next = filter::Next {
            target,
            output_filters,
            input_filters,
            call: next_node.call.clone(),
            location: filter::NextLocation::Source,
        };
        let response = next.call(req, self).await;
        Ok(response)
    }

    pub async fn call(&mut self, req: DynRequest, port: NodePort) -> DynResponse {
        match self.call_next_with_error(req, port).await {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("Flow error: {}", e);
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
            let response = (entry_node.call.clone())(req, &mut context).await;
            Ok(response)
        })
    }
}
