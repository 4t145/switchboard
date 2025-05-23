use std::collections::HashMap;

use http::request::Parts;

use crate::object::class::SbhClass;

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

impl SbhClass for Host {
    type Error = serde_json::Error;
    type Type = SharedRouter;
    fn name(&self) -> crate::object::class::ObjectClassName {
        crate::object::class::ObjectClassName::std("host")
    }
    fn construct(&self, config: &str) -> Result<Self::Type, Self::Error> {
        let map: HashMap<String, Route> = serde_json::from_str(config)?;
        Ok(SharedRouter::new(HostRouter { map }))
    }
}
