pub mod path;
pub mod rule;
use std::{collections::BTreeMap, sync::Arc};

use crate::{hostname::HostnameTree, path::PathTree};

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]

pub struct RouterSerde<T> {
    #[serde(default = "BTreeMap::new")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub hostname: BTreeMap<String, path::PathTreeSerdeMapStyle<T>>,
}

impl <T> Default for RouterSerde<T> {
    fn default() -> Self {
        Self {
            hostname: BTreeMap::new(),
        }
    }
}

impl<T> TryInto<crate::Router<T>> for RouterSerde<T>
where
    T: Clone,
{
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<crate::Router<T>, Self::Error> {
        let mut hostname_tree = HostnameTree::new();
        for (hostname, tree) in self.hostname {
            let tree: PathTree<T> = tree.try_into()?;
            let hostname: Arc<str> = Arc::from(hostname);
            hostname_tree.set(&hostname.clone(), (hostname, tree));
        }
        Ok(crate::Router { hostname_tree })
    }
}
