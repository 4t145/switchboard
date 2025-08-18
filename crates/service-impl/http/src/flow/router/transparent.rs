
use super::{Router, NodePort};

pub struct Transparent;

impl Router for Transparent {
    fn route(&self, _req: &mut http::request::Parts) -> NodePort {
        NodePort::Default
    }
}
