pub mod class;
// pub mod orchestration;
pub mod registry;

use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    fmt::Display,
    ops::Deref,
    str::FromStr,
    sync::Arc,
};

use class::ClassId;
use serde::{Deserialize, Serialize};

use crate::{
    flow::{
        filter::Filter,
        node::{Node, NodeId, NodeInterface},
    },
    instance::class::ClassData,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum InstanceType {
    Node,
    Filter,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceId(Arc<str>);

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InstanceData {
    pub id: InstanceId,
    pub class: ClassId,
    pub r#type: InstanceType,
    pub config: serde_json::Value,
}

pub enum InstanceValue {
    Node(Node),
    Filter(Filter),
}

pub struct Instance {
    pub id: InstanceId,
    pub class: ClassId,
    pub value: InstanceValue,
    pub config: serde_json::Value,
}
