use std::{collections::HashMap, convert::Infallible};

use http::request::Parts;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::flow::{
    node::{NodeClass, NodeOutput, NodePort},
    router::{RouterNode, WithRoutes},
};

use super::Router;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

    fn id(&self) -> crate::instance::class::ClassId {
        crate::instance::class::ClassId::std("host-match")
    }
}
