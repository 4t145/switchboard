use std::collections::{BTreeMap, HashMap, HashSet};

use crate::service::dynamic::SharedService;

use super::{
    Node, NodeId,
    class::{KindLayer, KindRouter, KindService, NodeClass, NodeClassName},
};

pub struct NodeClassRegistry {
    pub router: HashMap<NodeClassName, NodeClass<KindRouter>>,
    pub layer: HashMap<NodeClassName, NodeClass<KindLayer>>,
    pub service: HashMap<NodeClassName, NodeClass<KindService>>,
}

pub struct NodeRegistry {
    pub router: HashMap<NodeId, Node<KindRouter>>,
    pub layer: HashMap<NodeId, Node<KindLayer>>,
    pub service: HashMap<NodeId, Node<KindService>>,
}

pub enum OrchestrationError {
    UndefinedClass(NodeClassName),
    MakeError {
        class_name: NodeClassName,
        node_id: NodeId,
        error: anyhow::Error,
    },
}

pub fn orchestration(
    reg_class: &NodeClassRegistry,
    reg_node: &NodeRegistry,
) -> Result<HashMap<NodeId, SharedService>, OrchestrationError> {
    let mut entry_points = HashMap::new();
    // 1. build reverse links

    let mut reverse_link_map = HashMap::<NodeId, HashSet<NodeId>>::new();
    for (node_id, node) in reg_node.router.iter() {
        reverse_link_map
            .entry(node_id.clone())
            .or_default()
            .extend(node.link.table.values().cloned())
    }
    for (node_id, node) in reg_node.layer.iter() {
        reverse_link_map
            .entry(node_id.clone())
            .or_default()
            .insert(node.link.next.clone());
    }

    // 2. find out all terminated nodes
    let mut built_services = HashMap::new();
    for (id, node) in reg_node.service.iter() {
        // find class
        let class = reg_class
            .service
            .get(&node.class)
            .ok_or(OrchestrationError::UndefinedClass(node.class.clone()))?;

        // make service
        let service =
            class
                .maker
                .make(&node.config)
                .map_err(|error| OrchestrationError::MakeError {
                    class_name: node.class.clone(),
                    node_id: node.id.clone(),
                    error,
                })?;

        // storage
        built_services.insert(id.clone(), service);
    }


    todo!();
    Ok(entry_points)
}
