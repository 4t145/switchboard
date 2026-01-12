use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use switchboard_custom_config::SerdeValue;
pub mod consts;
#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]

pub struct Config<Cfg = SerdeValue> {
    pub flow: FlowConfig<Cfg>,
    #[serde(default)]
    pub server: ServerConfig,
}

impl<Cfg> Default for Config<Cfg> {
    fn default() -> Self {
        Self {
            flow: FlowConfig::new(),
            server: ServerConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]

#[serde(default)]
pub struct ServerConfig {
    pub version: HttpVersion,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            version: HttpVersion::Auto,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "lowercase")]
pub enum HttpVersion {
    Http1,
    #[serde(alias = "h2")]
    Http2,
    #[default]
    Auto,
}

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]

pub struct FlowConfig<Cfg = SerdeValue> {
    pub entrypoint: NodeTarget,
    #[serde(alias = "instance")]
    #[serde(default = "BTreeMap::new", skip_serializing_if = "BTreeMap::is_empty")]
    pub instances: BTreeMap<InstanceId, InstanceData<Cfg>>,
    #[serde(alias = "node")]
    #[serde(default = "BTreeMap::new", skip_serializing_if = "BTreeMap::is_empty")]
    pub nodes: BTreeMap<InstanceId, InstanceDataWithoutType<Cfg>>,
    #[serde(alias = "filter")]
    #[serde(default = "BTreeMap::new", skip_serializing_if = "BTreeMap::is_empty")]
    pub filters: BTreeMap<InstanceId, InstanceDataWithoutType<Cfg>>,
    #[serde(default)]
    pub options: FlowOptions,
}

impl<Cfg> FlowConfig<Cfg> {
    pub fn new() -> Self {
        Self {
            entrypoint: NodeTarget::from("inbound"),
            instances: BTreeMap::new(),
            nodes: BTreeMap::new(),
            filters: BTreeMap::new(),
            options: FlowOptions::default(),
        }
    }
}

pub type FlowConfigWithLink = FlowConfig<switchboard_custom_config::Link>;

#[derive(Debug, Clone, Serialize, Deserialize, Default, bincode::Encode, bincode::Decode)]

pub struct FlowOptions {
    pub max_loop: Option<u32>,
}

use std::{collections::BTreeMap, convert::Infallible, fmt::Display, str::FromStr, sync::Arc};

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode, Default)]

pub enum InstanceType {
    #[default]
    Node,
    Filter,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
    bincode::Encode,
    bincode::Decode,
)]
#[serde(transparent)]
/// instance id can only contain alphanumeric characters, hyphens, dots, and underscores
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

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct InstanceData<Cfg = SerdeValue> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub class: ClassId,
    #[serde(default)]
    pub r#type: InstanceType,
    pub config: Cfg,
}

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct InstanceDataWithoutType<Cfg = SerdeValue> {
    pub name: Option<String>,
    pub class: ClassId,
    pub config: Cfg,
}

impl<Cfg> InstanceDataWithoutType<Cfg> {
    pub fn with_type(self, r#type: InstanceType) -> InstanceData<Cfg> {
        InstanceData {
            name: self.name,
            class: self.class,
            r#type,
            config: self.config,
        }
    }
}

pub type NodeId = InstanceId;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Default, bincode::Encode, bincode::Decode, PartialOrd, Ord,
)]
pub enum NodePort {
    Named(Arc<str>),
    #[default]
    Default,
}

impl From<&str> for NodePort {
    fn from(s: &str) -> Self {
        if s == "$default" {
            NodePort::Default
        } else {
            NodePort::Named(Arc::from(s))
        }
    }
}

impl From<String> for NodePort {
    fn from(s: String) -> Self {
        if s == "$default" {
            NodePort::Default
        } else {
            NodePort::Named(Arc::from(s))
        }
    }
}



impl NodePort {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self::from(name.as_ref())
    }
    pub fn is_default(&self) -> bool {
        matches!(self, NodePort::Default)
    }
    pub fn as_str(&self) -> &str {
        match self {
            NodePort::Named(name) => name,
            NodePort::Default => "$default",
        }
    }
}

impl FromStr for NodePort {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "$default" {
            Ok(NodePort::Default)
        } else {
            Ok(NodePort::Named(Arc::from(s)))
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
        Self::from_str(&s).map_err(serde::de::Error::custom)
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

#[derive(Clone, Serialize, Deserialize, Debug, bincode::Encode, bincode::Decode)]
pub struct NodeInterface {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub inputs: BTreeMap<NodePort, NodeInput>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub outputs: BTreeMap<NodePort, NodeOutput>,
}

#[derive(Clone, Serialize, Deserialize, Debug, bincode::Encode, bincode::Decode)]
pub struct NodeInput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<FilterReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct NodeOutput {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<FilterReference>,
    pub target: NodeTarget,
}

impl NodeInterface {
    pub fn new(
        inputs: BTreeMap<NodePort, NodeInput>,
        outputs: BTreeMap<NodePort, NodeOutput>,
    ) -> Self {
        Self { inputs, outputs }
    }
    pub fn with_default_input(outputs: BTreeMap<NodePort, NodeOutput>) -> Self {
        Self {
            inputs: BTreeMap::from_iter([(
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub struct NodeTarget {
    pub id: NodeId,
    pub port: NodePort,
}

impl From<&str> for NodeTarget {
    fn from(s: &str) -> Self {
        Self::from_str(s).expect("Infallible")
    }
}

impl From<String> for NodeTarget {
    fn from(s: String) -> Self {
        Self::from_str(&s).expect("Infallible")
    }
}

impl From<NodeId> for NodeTarget {
    fn from(id: NodeId) -> Self {
        Self {
            id,
            port: NodePort::Default,
        }
    }
}

impl Serialize for NodeTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.port == NodePort::Default {
            serializer.serialize_str(self.id.0.as_ref())
        } else {
            serializer.serialize_str(&format!("{}:{}", self.id, self.port.as_str()))
        }
    }
}

impl<'de> Deserialize<'de> for NodeTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for NodeTarget {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((id, port)) = s.split_once(':') {
            Ok(NodeTarget {
                id: NodeId::new(id),
                port: NodePort::from_str(port)?,
            })
        } else {
            Ok(NodeTarget {
                id: NodeId::new(s),
                port: NodePort::Default,
            })
        }
    }
}

impl Display for NodeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.id, self.port)
    }
}

pub type FilterId = InstanceId;

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(transparent)]
pub struct FilterReference {
    pub id: FilterId,
    // pub call: Arc<FilterFn>,
}

impl From<InstanceId> for FilterReference {
    fn from(id: InstanceId) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct ClassId {
    pub namespace: Option<String>,
    pub name: String,
}

impl serde::Serialize for ClassId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ClassId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl ClassId {
    pub fn std(name: impl Into<String>) -> Self {
        Self {
            namespace: None,
            name: name.into(),
        }
    }
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            name: name.into(),
        }
    }
}

impl FromStr for ClassId {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((ns, name)) = s.split_once('.') {
            Ok(ClassId {
                namespace: Some(ns.to_string()),
                name: name.to_string(),
            })
        } else {
            Ok(ClassId {
                namespace: None,
                name: s.to_string(),
            })
        }
    }
}

impl std::fmt::Display for ClassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}.{}", namespace, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]

#[derive(bincode::Encode, bincode::Decode)]
pub struct ClassMeta {
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
}

impl Default for ClassMeta {
    fn default() -> Self {
        Self::from_env()
    }
}
impl ClassMeta {
    pub fn from_env() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: Some(env!("CARGO_PKG_DESCRIPTION").to_string()),
            author: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            license: Some(env!("CARGO_PKG_LICENSE").to_string()),
            repository: Some(env!("CARGO_PKG_REPOSITORY").to_string()),
            homepage: Some(env!("CARGO_PKG_HOMEPAGE").to_string()),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]

// #[derive(bincode::Encode, bincode::Decode)]
pub struct ClassData {
    pub id: ClassId,
    pub meta: ClassMeta,
    pub instance_type: InstanceType,
    // pub config_schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]

pub struct WithOutputs<C> {
    #[serde(flatten)]
    pub config: C,
    pub output: BTreeMap<NodePort, NodeOutput>,
}
