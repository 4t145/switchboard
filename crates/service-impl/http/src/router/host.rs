use std::collections::HashMap;

use http::request::Parts;
use typeshare::typeshare;

use crate::instance::class::SbhClass;

use super::{Route, Router, SharedRouter};

pub struct HostRouter {
    map: HashMap<String, Route>,
}

impl Router for HostRouter {
    fn route(&self, req: &mut Parts) -> Route {
        req.headers
            .get(http::header::HOST)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| self.map.get(h))
            .cloned()
            .unwrap_or(Route::Fallback)
    }
}

pub struct Host;
#[typeshare]
pub type HostRouterConfig = HashMap<String, Route>;
impl SbhClass for Host {
    type Error = serde_json::Error;
    type Type = SharedRouter;
    type Config = HostRouterConfig;
    fn id(&self) -> crate::instance::class::ClassId {
        crate::instance::class::ClassId::std("host")
    }
    fn construct(&self, config: HostRouterConfig) -> Result<Self::Type, Self::Error> {
        Ok(SharedRouter::new(HostRouter { map: config }))
    }
}
