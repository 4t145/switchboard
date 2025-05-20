use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::router::Route;

use super::NodeId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouterLink {
    pub table: BTreeMap<Route, NodeId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayerLink {
    pub next: NodeId,
}
