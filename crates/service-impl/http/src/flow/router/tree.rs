use http::request::Parts;

use switchboard_http_router::{Router as TreeRouter, serde::RouterSerde};
use switchboard_model::services::http::{ClassId, WithRoutes};

use crate::flow::{node::NodeClass, router::RouterNode};

use super::{NodePort, Router};

pub struct PathRouter {
    pub router: TreeRouter<NodePort>,
}

pub struct PathRouterEndpoint {
    pub route: NodePort,
}

impl Router for PathRouter {
    fn route(&self, req: &mut Parts) -> NodePort {
        let match_result = self.router.match_request_parts(req);
        match match_result {
            Ok(matched) => matched.get_data().clone(),
            Err(_) => NodePort::Default,
        }
    }
}

pub struct PathMatch;

#[derive(Debug, thiserror::Error)]
pub enum PathMatchConstructError {
    #[error("build router error: {0}")]
    BuildError(#[from] switchboard_http_router::error::BuildError),
}

impl NodeClass for PathMatch {
    type Config = WithRoutes<RouterSerde<NodePort>>;
    type Error = PathMatchConstructError;
    type Node = RouterNode<PathRouter>;
    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        let router_config = config.router_config;
        let router: TreeRouter<NodePort> = router_config.try_into()?;
        Ok(RouterNode::new(config.routes, PathRouter { router }))
    }

    fn id(&self) -> ClassId {
        ClassId::std("path-match")
    }
}
