
use super::{Router, NodePort};

pub struct Transparent;

impl Router for Transparent {
    fn route(&self, _req: &mut http::request::Parts) -> NodePort {
        NodePort::Default
    }
}

// impl SbhClass for Transparent {
//     type Type = SharedRouter;
//     type Error = serde_json::Error;
//     type Config = ();
//     fn id(&self) -> crate::instance::class::ClassId {
//         crate::instance::class::ClassId::std("transparent")
//     }
//     fn construct(&self, _config: ()) -> Result<Self::Type, Self::Error> {
//         Ok(SharedRouter::new(Transparent))
//     }
// }
