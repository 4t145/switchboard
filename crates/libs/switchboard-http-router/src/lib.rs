use crate::utils::hostname::try_extract_hostname;

pub mod error;
pub mod hostname;
pub mod path;
pub mod rule;
pub mod utils;
pub mod serde;
#[derive(Debug, Clone, Default)]
pub struct Router<T: Clone> {
    pub hostname_tree: hostname::HostnameTree<path::PathTree<T>>,
}

impl<T: Clone> Router<T> {
    pub fn new() -> Self {
        Self {
            hostname_tree: hostname::HostnameTree::new(),
        }
    }

    pub fn match_request_parts<'p>(
        &self,
        parts: &'p http::request::Parts,
    ) -> Result<path::PathTreeMatched<'p, T>, error::Error> {
        let host = try_extract_hostname(parts)?;
        if let Some(tree) = self.hostname_tree.get(host) {
            tree.match_request_parts(parts)
                .ok_or(error::Error::NoMatchRoute)
        } else {
            Err(error::Error::HostNotFound)
        }
    }
}
