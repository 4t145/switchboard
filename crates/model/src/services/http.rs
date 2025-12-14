use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};

pub mod rule_router;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub flow: FlowConfig,
    pub server: ServerConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub version: HttpVersion,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HttpVersion {
    Http1,
    #[serde(alias = "h2")]
    Http2,
    #[default]
    Auto,
}


#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub struct FlowConfig {
    pub entrypoint: NodeTarget,
    pub instances: HashMap<InstanceId, InstanceData>,
    pub options: FlowOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all="camelCase")]
pub struct FlowOptions {
    pub max_loop: Option<u32>,
}

use std::{collections::HashMap, fmt::Display, sync::Arc};


#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum InstanceType {
    Node,
    Filter,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct InstanceId(pub(crate) Arc<str>);

impl InstanceId {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(Arc::from(s.as_ref()))
    }
}

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct InstanceData {
    pub name: Option<String>,
    pub class: ClassId,
    pub r#type: InstanceType,
    pub config: serde_json::Value,
}

pub type NodeId = InstanceId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum NodePort {
    Named(Arc<str>),
    #[default]
    Default,
}

impl NodePort {
    pub fn as_str(&self) -> &str {
        match self {
            NodePort::Named(name) => name,
            NodePort::Default => "$default",
        }
    }
}

impl JsonSchema for NodePort {
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> Schema {
        String::json_schema(generator)
    }
    fn schema_name() -> std::borrow::Cow<'static, str> {
        String::schema_name()
    }
    fn inline_schema() -> bool {
        String::inline_schema()
    }
    fn schema_id() -> std::borrow::Cow<'static, str> {
        String::schema_id()
    }
}

impl Serialize for NodePort {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            NodePort::Named(name) => serializer.serialize_str(name),
            NodePort::Default => serializer.serialize_str("$default"),
        }
    }
}

impl<'de> Deserialize<'de> for NodePort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "$default" {
            Ok(NodePort::Default)
        } else {
            Ok(NodePort::Named(Arc::from(s)))
        }
    }
}

impl std::fmt::Display for NodePort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodePort::Named(name) => write!(f, "{}", name),
            NodePort::Default => write!(f, "$default"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NodeInterface {
    pub inputs: HashMap<NodePort, NodeInput>,
    pub outputs: HashMap<NodePort, NodeOutput>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NodeInput {
    pub filters: Vec<FilterReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NodeOutput {
    pub filters: Vec<FilterReference>,
    pub target: NodeTarget,
}

impl NodeInterface {
    pub fn new(
        inputs: HashMap<NodePort, NodeInput>,
        outputs: HashMap<NodePort, NodeOutput>,
    ) -> Self {
        Self { inputs, outputs }
    }
    pub fn with_default_input(outputs: HashMap<NodePort, NodeOutput>) -> Self {
        Self {
            inputs: HashMap::from_iter([(
                NodePort::Default,
                NodeInput {
                    filters: Vec::new(),
                },
            )]),
            outputs,
        }
    }
    pub fn service() -> Self {
        Self::with_default_input(Default::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct NodeTarget {
    pub id: NodeId,
    pub port: NodePort,
}

impl Display for NodeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.id, self.port)
    }
}

pub type FilterId = InstanceId;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FilterReference {
    pub id: FilterId,
    // pub call: Arc<FilterFn>,
}


#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, JsonSchema)]
pub struct ClassId {
    pub namespace: Option<String>,
    pub name: String,
}
