use std::collections::HashMap;

use http::request::Parts;
use typeshare::typeshare;

use crate::flow::node::NodePort;

use super::Router;

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

pub struct Host;
#[typeshare]
pub type HostRouterConfig = HashMap<String, NodePort>;
// impl RouterNode for Host {
//     type Error = serde_json::Error;
//     type Type = SharedRouter;
//     type Config = HostRouterConfig;
//     fn id(&self) -> crate::instance::class::ClassId {
//         crate::instance::class::ClassId::std("host")
//     }
//     fn construct(&self, config: HostRouterConfig) -> Result<Self::Type, Self::Error> {
//         Ok(SharedRouter::new(HostRouter { map: config }))
//     }
// }
