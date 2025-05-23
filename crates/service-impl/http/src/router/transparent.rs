use crate::object::class::SbhClass;

use super::{Router, SharedRouter};

pub struct Transparent;

impl Router for Transparent {
    fn route(&self, _req: &mut http::request::Parts) -> super::Route {
        super::Route::Fallback
    }
}

impl SbhClass for Transparent {
    type Type = SharedRouter;
    type Error = serde_json::Error;
    fn name(&self) -> crate::object::class::ObjectClassName {
        crate::object::class::ObjectClassName::std("transparent")
    }
    fn construct(&self, _config: &str) -> Result<Self::Type, Self::Error> {
        Ok(SharedRouter::new(Transparent))
    }
}
