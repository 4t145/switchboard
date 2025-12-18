use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use switchboard_custom_config::CustomConfig;

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct Config<Cfg = CustomConfig> {
    pub flow: FlowConfig<Cfg>,
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct FlowConfig<Cfg = CustomConfig> {
    pub entrypoint: NodeTarget,
    pub instances: BTreeMap<InstanceId, InstanceData<Cfg>>,
    #[serde(default)]
    pub options: FlowOptions,
}

pub type FlowConfigWithLink = FlowConfig<switchboard_custom_config::Link>;

#[derive(Debug, Clone, Serialize, Deserialize, Default, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct FlowOptions {
    pub max_loop: Option<u32>,
}

use std::{
    collections::{BTreeMap, HashMap},
    convert::Infallible,
    fmt::Display,
    str::FromStr,
    sync::Arc,
};

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub enum InstanceType {
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
pub struct InstanceData<Cfg = CustomConfig> {
    pub name: Option<String>,
    pub class: ClassId,
    pub r#type: InstanceType,
    pub config: Cfg,
}

pub type NodeId = InstanceId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, bincode::Encode, bincode::Decode)]
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

#[derive(Clone, Serialize, Deserialize, Debug, bincode::Encode, bincode::Decode)]
pub struct NodeInterface {
    pub inputs: HashMap<NodePort, NodeInput>,
    pub outputs: HashMap<NodePort, NodeOutput>,
}

#[derive(Clone, Serialize, Deserialize, Debug, bincode::Encode, bincode::Decode)]
pub struct NodeInput {
    pub filters: Vec<FilterReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
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

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct NodeTarget {
    pub id: NodeId,
    #[serde(default)]
    pub port: NodePort,
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
// #[derive(bincode::Encode, bincode::Decode)]
pub struct ClassData {
    pub id: ClassId,
    pub meta: ClassMeta,
    pub instance_type: InstanceType,
    // pub config_schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(rename_all = "camelCase")]
pub struct WithRoutes<C> {
    #[serde(flatten)]
    pub config: C,
    pub output: HashMap<NodePort, NodeOutput>,
}
