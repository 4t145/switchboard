use std::fmt::Debug;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::{
    link::{LayerLink, RouterLink},
    maker::{LayerMaker, NodeMaker, RouterMaker, ServiceMaker},
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct NodeClassName {
    pub namespace: Option<String>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NodeClassKindEnum {
    Layer,
    Service,
    Router,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeClassMeta {
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub document: Option<String>,
    pub schema: Option<schemars::schema::RootSchema>,
    pub kind: NodeClassKindEnum,
}

pub struct NodeClass<K: NodeClassKind> {
    pub name: NodeClassName,
    pub meta: NodeClassMeta,
    pub maker: K::Maker,
}

pub trait NodeClassKind {
    type Maker;
    type Link: Serialize + DeserializeOwned + Clone + Debug;
    fn kind() -> NodeClassKindEnum;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum KindService {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum KindLayer {}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub enum KindRouter {}

impl NodeClassKind for KindService {
    type Link = ();
    type Maker = ServiceMaker;
    fn kind() -> NodeClassKindEnum {
        NodeClassKindEnum::Service
    }
}

impl NodeClassKind for KindLayer {
    type Link = LayerLink;
    type Maker = LayerMaker;
    fn kind() -> NodeClassKindEnum {
        NodeClassKindEnum::Layer
    }
}

impl NodeClassKind for KindRouter {
    type Link = RouterLink;
    type Maker = RouterMaker;
    fn kind() -> NodeClassKindEnum {
        NodeClassKindEnum::Router
    }
}
