use std::{collections::HashMap, convert::Infallible};

use http::request::Parts;
use serde::{Deserialize, Serialize};


use crate::flow::{node::NodeClass, router::RouterNode};
use switchboard_model::services::http::{NodeOutput, NodePort, WithRoutes, ClassId};
use super::Router;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostRouter {
    map: HashMap<String, NodePort>,
}

impl Router for HostRouter {
    fn route(&self, req: &mut Parts) -> NodePort {
        req.headers
            .get(http::header::HOST)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| self.map.get(h))
            .cloned()
            .unwrap_or(NodePort::Default)
    }
}

pub struct HostMatch;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostRouterConfig {
    pub map: HashMap<String, NodePort>,
    pub routes: HashMap<NodePort, NodeOutput>,
}

impl NodeClass for HostMatch {
    type Config = WithRoutes<HostRouter>;
    type Error = Infallible;
    type Node = RouterNode<HostRouter>;
    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        Ok(RouterNode::new(config.routes, config.router_config))
    }

    fn id(&self) -> ClassId {
        ClassId::std("host-match")
    }
}
