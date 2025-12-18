use http::request::Parts;

use switchboard_http_router::{Router as TreeRouterInner, serde::RouterSerde};
use switchboard_model::services::http::{ClassId, WithRoutes};

use crate::flow::{node::NodeClass, router::RouterNode};
pub type TreeRouterMatched = switchboard_http_router::RouterMatched<NodePort>;

use super::{NodePort, Router};

pub struct RouterRouter {
    pub router: TreeRouterInner<NodePort>,
    pub options: TreeRouterOptions,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct RouterRouterConfig {
    #[serde(flatten)]
    pub router: RouterSerde<NodePort>,
    #[serde(default)]
    pub options: TreeRouterOptions,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode, Default,
)]
#[serde(default)]
pub struct TreeRouterOptions {
    // #[serde(default)]
    // pub preserve_host: bool,
}

impl TreeRouterOptions {
    pub fn process_request_parts(&self, req: &mut Parts) {
        {
            req
        };
    }
}

pub struct PathRouterEndpoint {
    pub route: NodePort,
}

impl Router for RouterRouter {
    fn route(&self, req: &mut Parts) -> NodePort {
        let match_result = self.router.match_request_parts(req);
        match match_result {
            Ok(matched) => {
                self.options.process_request_parts(req);
                // store matched data into extensions
                let data = matched.get_data().clone();
                req.extensions.insert(matched);
                data
            }
            Err(_) => NodePort::Default,
        }
    }
}

pub struct TreeRouterClass;

#[derive(Debug, thiserror::Error)]
pub enum RouterRouterConstructError {
    #[error("build router error: {0}")]
    BuildError(#[from] switchboard_http_router::error::BuildError),
}

impl NodeClass for TreeRouterClass {
    type Config = WithRoutes<RouterRouterConfig>;
    type Error = RouterRouterConstructError;
    type Node = RouterNode<RouterRouter>;
    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        let RouterRouterConfig { router, options } = config.config;
        let router: TreeRouterInner<NodePort> = router.try_into()?;
        Ok(RouterNode::new(
            config.output,
            RouterRouter { router, options },
        ))
    }

    fn id(&self) -> ClassId {
        ClassId::std("router")
    }
}
