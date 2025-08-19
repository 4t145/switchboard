use std::{collections::HashMap, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    flow::{
        node::{NodePort, NodeTarget}, Flow, FlowOptions
    },
    instance::{self, class::registry::ClassRegistryError, InstanceData, InstanceId},
};

use crate::instance::class::registry::ClassRegistry;

#[derive(Debug, thiserror::Error)]
pub enum FlowBuildError {
    #[error("Class registry error")]
    ClassRegistryError(#[from] ClassRegistryError),
    #[error("Invalid flow {}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "))]
    InvalidFlow(Vec<FlowCheckError>),
}

#[derive(Debug)]
pub struct FilterReferenceLocation {
    pub node_id: InstanceId,
    pub interface_kind: InterfaceKind,
    pub port: NodePort,
    pub index: usize,
}

#[derive(Debug)]
pub struct TargetLocation {
    pub node_id: InstanceId,
    pub port: NodePort,
}

#[derive(Debug)]
pub enum InterfaceKind {
    Input,
    Output,
}

#[derive(Debug)]
pub enum FlowCheckError {
    FilterNotFound {
        filter: InstanceId,
        location: FilterReferenceLocation,
    },
    NodeTargetNotFound {
        target: NodeTarget,
        location: TargetLocation,
    },
    EntrypointNotFound {
        entrypoint: NodeTarget,
    },
}

impl std::fmt::Display for FlowCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowCheckError::FilterNotFound { filter, location } => write!(
                f,
                "Filter `{}` not found at {}:{:?}[{}].filter[{}]",
                filter, location.node_id, location.interface_kind, location.port, location.index
            ),
            FlowCheckError::NodeTargetNotFound { target, location } => write!(
                f,
                "Node target `{}` not found at {}:{}",
                target.id, location.node_id, location.port
            ),
            FlowCheckError::EntrypointNotFound { entrypoint } => {
                write!(f, "Entrypoint `{}` not found", entrypoint)
            }
        }
    }
}

#[typeshare]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FlowConfig {
    entrypoint: NodeTarget,
    instances: HashMap<InstanceId, InstanceData>,
    options: FlowOptions,
}

impl Flow {
    pub fn check(&self) -> Vec<FlowCheckError> {
        let mut check_errors = vec![];
        // check if entrypoint exist:
        if !self
            .nodes
            .get(&self.entrypoint.id)
            .is_some_and(|entry| entry.interface.inputs.contains_key(&NodePort::Default))
        {
            check_errors.push(FlowCheckError::EntrypointNotFound {
                entrypoint: self.entrypoint.clone(),
            });
        }
        for (node_id, node) in self.nodes.iter() {
            for (index, (port, output)) in node.interface.outputs.iter().enumerate() {
                // check if filter exists
                for filter in &output.filters {
                    if !self.filters.contains_key(&filter.id) {
                        check_errors.push(FlowCheckError::FilterNotFound {
                            filter: filter.id.clone(),
                            location: FilterReferenceLocation {
                                node_id: node_id.clone(),
                                interface_kind: InterfaceKind::Output,
                                port: port.clone(),
                                index,
                            },
                        });
                    }
                }
                // check if target exists
                if !self
                    .nodes
                    .get(&output.target.id)
                    .is_some_and(|target_node| {
                        target_node
                            .interface
                            .inputs
                            .contains_key(&output.target.port)
                    })
                {
                    check_errors.push(FlowCheckError::NodeTargetNotFound {
                        target: output.target.clone(),
                        location: TargetLocation {
                            node_id: node_id.clone(),
                            port: port.clone(),
                        },
                    });
                }
            }
            for (index, (port, input)) in node.interface.inputs.iter().enumerate() {
                // check if filter exists
                for filter in &input.filters {
                    if !self.filters.contains_key(&filter.id) {
                        check_errors.push(FlowCheckError::FilterNotFound {
                            filter: filter.id.clone(),
                            location: FilterReferenceLocation {
                                node_id: node_id.clone(),
                                interface_kind: InterfaceKind::Input,
                                port: port.clone(),
                                index,
                            },
                        });
                    }
                }
            }
        }
        // check loops

        check_errors
    }

    pub fn build(
        config: FlowConfig,
        class_registry: &ClassRegistry,
    ) -> Result<Self, FlowBuildError> {
        let mut filters = HashMap::new();
        let mut nodes = HashMap::new();
        for (id, instance) in config.instances {
            let instance = class_registry.construct(instance.class, instance.config)?;
            match instance {
                instance::InstanceValue::Node(node) => {
                    nodes.insert(id, node);
                }
                instance::InstanceValue::Filter(filter) => {
                    filters.insert(id, filter);
                }
            }
        }
        let flow = Flow {
            nodes: Arc::new(nodes),
            filters: Arc::new(filters),
            entrypoint: config.entrypoint,
        };
        let errors = flow.check();
        if errors.is_empty() {
            return Ok(flow);
        } else {
            return Err(FlowBuildError::InvalidFlow(errors));
        }
    }
}
