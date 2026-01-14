use std::sync::Arc;

use crate::utils::hostname::try_extract_hostname;

pub mod error;
pub mod hostname;
pub mod path;
pub mod rule;
pub mod serde;
pub mod utils;
#[derive(Debug, Clone, Default)]
pub struct Router<T: Clone> {
    pub hostname_tree: hostname::HostnameTree<(Arc<str>, path::PathTree<T>)>,
}

#[derive(Debug, Clone)]
pub struct RouterMatched<T: Clone> {
    pub hostname: Arc<str>,
    pub path_tree_matched: path::PathTreeMatched<T>,
}

impl<T: Clone> Router<T> {
    pub fn new() -> Self {
        Self {
            hostname_tree: hostname::HostnameTree::new(),
        }
    }
    pub fn set(&mut self, hostname: impl AsRef<str>, tree: path::PathTree<T>) {
        let hostname: Arc<str> = Arc::from(hostname.as_ref());
        self.hostname_tree.set(&hostname.clone(), (hostname, tree));
    }
    pub fn match_request_parts(
        &self,
        parts: &http::request::Parts,
    ) -> Result<RouterMatched<T>, error::Error> {
        let host = try_extract_hostname(parts)?;
        if let Some(tree) = self.hostname_tree.get(host) {
            let tree_matched = tree
                .1
                .match_request_parts(parts)
                .ok_or(error::Error::NoMatchRoute)?;
            Ok(RouterMatched {
                hostname: tree.0.clone(),
                path_tree_matched: tree_matched,
            })
        } else {
            Err(error::Error::HostNotFound)
        }
    }
}

impl<T: Clone> RouterMatched<T> {
    pub fn get_data(&self) -> &T {
        self.path_tree_matched.get_data()
    }
}
