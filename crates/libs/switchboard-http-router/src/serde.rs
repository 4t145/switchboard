pub mod path;
pub mod rule;
use std::collections::BTreeMap;

use crate::{hostname::HostnameTree, path::PathTree};

#[derive(
    Debug, Clone, Default, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]
#[serde(rename_all = "camelCase")]
pub struct RouterSerde<T> {
    #[serde(default = "BTreeMap::new")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub hostname_tree: BTreeMap<String, path::PathTreeSerde<T>>,
}

impl<T> TryInto<crate::Router<T>> for RouterSerde<T>
where
    T: Clone,
{
    type Error = crate::error::BuildError;

    fn try_into(self) -> Result<crate::Router<T>, Self::Error> {
        let mut hostname_tree = HostnameTree::new();
        for (hostname, tree) in self.hostname_tree {
            let tree: PathTree<T> = tree.try_into()?;
            hostname_tree.set(&hostname, tree);
        }
        Ok(crate::Router { hostname_tree })
    }
}
