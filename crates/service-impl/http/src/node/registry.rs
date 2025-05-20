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
    NodeNotFound(NodeId),
    MakeError {
        class_name: NodeClassName,
        node_id: NodeId,
        error: anyhow::Error,
    },
}

pub struct Orchestration<'r> {
    reg_class: &'r NodeClassRegistry,
    reg_node: &'r NodeRegistry,
    entry_points: HashSet<NodeId>,
    built_services: HashMap<NodeId, SharedService>,
    link_map: HashMap<NodeId, HashSet<NodeId>>,
    reverse_link_map: HashMap<NodeId, HashSet<NodeId>>,
}

impl<'r>  Orchestration<'r> {
    pub fn new(
        reg_class: &'r NodeClassRegistry,
        reg_node: &'r NodeRegistry,
    ) -> Self 
    {
        Orchestration {
            reg_class,
            reg_node,
            entry_points: HashSet::new(),
            built_services: HashMap::new(),
            link_map: HashMap::new(),
            reverse_link_map: HashMap::new(),
        }
    }

    pub fn build_links(&mut self) {
        for (node_id, node) in self.reg_node.router.iter() {
            self.link_map
                .entry(node_id.clone())
                .or_default()
                .extend(node.link.table.values().cloned());
            for target in node.link.table.values() {
                self.reverse_link_map
                    .entry(target.clone())
                    .or_default()
                    .insert(node_id.clone());
            }
        }

        for (node_id, node) in self.reg_node.layer.iter() {
            self.link_map
                .entry(node_id.clone())
                .or_default()
                .insert(node.link.next.clone());
            self.reverse_link_map
                .entry(node.link.next.clone())
                .or_default()
                .insert(node_id.clone());
        }
    }

    pub fn build_entry_points(&mut self) {
        self.reverse_link_map
            .iter()
            .filter(|(_, targets)| targets.is_empty())
            .for_each(|(node_id, _)| {
                self.entry_points.insert(node_id.clone());
            });
    }

    pub fn get_or_build_node(&mut self, node_id: &NodeId) -> Result<SharedService, OrchestrationError>{
        if let Some(service) = self.built_services.get(node_id) {
            return Ok(service.clone());
        } else if let Some(service) = self.reg_node.service.get(node_id) {
            // find class
            let class = self
                .reg_class
                .service
                .get(&service.class)
                .ok_or(OrchestrationError::UndefinedClass(service.class.clone()))?;
            // make service
            let service = class
                .maker
                .make(&service.config)
                .map_err(|error| OrchestrationError::MakeError {
                    class_name: service.class.clone(),
                    node_id: service.id.clone(),
                    error,
                })?;
            self.built_services.insert(node_id.clone(), service.clone());
            Ok(service)
        }  else if let Some(layer) = self.reg_node.layer.get(node_id) {
            // find class
            let class = self
                .reg_class
                .layer
                .get(&layer.class)
                .ok_or(OrchestrationError::UndefinedClass(layer.class.clone()))?;
            let inner = &layer
                .link
                .next;
            let inner_service = self.get_or_build_node(inner)?;
            // make service
            let layer = class
                .maker
                .make(&layer.config)
                .map_err(|error| OrchestrationError::MakeError {
                    class_name: layer.class.clone(),
                    node_id: layer.id.clone(),
                    error,
                })?;
            // link inner service
            
        } 
        else if let Some(router) = self.reg_node.router.get(node_id) {

        } 
        else {
            return Err(OrchestrationError::NodeNotFound(node_id.clone()));
        }
    }
}


pub fn orchestration(
    reg_class: &NodeClassRegistry,
    reg_node: &NodeRegistry,
) -> Result<HashMap<NodeId, SharedService>, OrchestrationError> {
    let mut entry_points = HashMap::new();
    let mut pending_router_nodes = HashSet::new();
    let mut built_services = HashMap::new();

    // 1. build reverse links
    let mut link_map = HashMap::<NodeId, HashSet<NodeId>>::new();
    let mut reverse_link_map = HashMap::<NodeId, HashSet<NodeId>>::new();
    for (node_id, node) in reg_node.router.iter() {
        link_map
            .entry(node_id.clone())
            .or_default()
            .extend(node.link.table.values().cloned());
        for target in node.link.table.values() {
            reverse_link_map
                .entry(target.clone())
                .or_default()
                .insert(node_id.clone());
        }
    }

    for (node_id, node) in reg_node.layer.iter() {
        link_map
            .entry(node_id.clone())
            .or_default()
            .insert(node.link.next.clone());
        reverse_link_map
            .entry(node.link.next.clone())
            .or_default()
            .insert(node_id.clone());
    }

    // 2. find out all entry points
    for (node_id, node) in reg_node.router.iter() {
        // find class
        let class = reg_class
            .router
            .get(&node.class)
            .ok_or(OrchestrationError::UndefinedClass(node.class.clone()))?;
    }
    // 2. find out all terminated nodes
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

    loop {
        for 
    }
    todo!();
    Ok(entry_points)
}
