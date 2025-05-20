pub mod class;
pub mod maker;
pub mod registry;
pub mod link;
use std::{collections::BTreeMap, ops::Deref, sync::Arc};

use class::{NodeClassKind, NodeClassName};
use maker::{LayerMaker, RouterMaker, ServiceMaker};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct NodeId(Arc<str>);

impl Deref for NodeId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl std::fmt::Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl NodeId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
    pub fn random() -> Self {
        Self(Arc::from(uuid::Uuid::new_v4().to_string()))
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Node<K: NodeClassKind> {
    pub id: NodeId,
    pub class: NodeClassName,
    pub link: K::Link,
    pub config: String,
}

