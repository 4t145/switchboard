pub mod class;

use std::sync::Arc;

use class::ClassId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::flow::{filter::Filter, node::Node};
#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]

pub enum InstanceType {
    Node,
    Filter,
}
#[typeshare(serialized_as = "String")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct InstanceId(pub(crate) Arc<str>);

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct InstanceData {
    pub name: Option<String>,
    pub class: ClassId,
    pub r#type: InstanceType,
    pub config: serde_json::Value,
}

pub enum InstanceValue {
    Node(Node),
    Filter(Filter),
}
