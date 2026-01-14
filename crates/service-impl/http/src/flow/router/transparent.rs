use super::{NodePort, Router};

pub struct Transparent;

impl Router for Transparent {
    fn route(&self, _req: &mut http::request::Parts) -> NodePort {
        NodePort::Default
    }
}
