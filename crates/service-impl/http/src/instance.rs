pub mod class;
use crate::flow::{filter::Filter, node::Node};

pub enum InstanceValue {
    Node(Node),
    Filter(Filter),
}
