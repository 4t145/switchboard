use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    layer::{Layer, dynamic::SharedLayer},
    router::{Route, SharedRouter},
    service::{dynamic::SharedService, router::RouterService},
};

use super::{
    ObjectId,
    class::{ObjectClassName, ObjectClassType},
    registry::{ObjectClassRegistry, ObjectRegistry},
};

#[derive(Clone)]
struct Constructed<C: ObjectClassType>(pub HashMap<ObjectId, C>);
impl<C: ObjectClassType> std::default::Default for Constructed<C> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

#[derive(Clone, Default)]
pub struct Orchestration {
    pub(crate) constructed_layer: Constructed<SharedLayer>,
    pub(crate) constructed_service: Constructed<SharedService>,
    pub(crate) constructed_router: Constructed<SharedRouter>,
    pub(crate) built_services: HashMap<ObjectId, SharedService>,
}

pub enum OrchestrationError {
    UndefinedClass(ObjectClassName),
    ObjectNotFound(ObjectId),
    NotYetConstructed(ObjectId),
    BuildingServiceOnLayer(ObjectId),
    LoopDetected {
        id: ObjectId,
        trace: Vec<ObjectId>,
    },
    MakeError {
        class_name: ObjectClassName,
        node_id: ObjectId,
        error: anyhow::Error,
    },
}
#[derive(Clone)]
pub struct OrchestrationContext<'a> {
    class_registry: &'a ObjectClassRegistry,
    object_registry: &'a ObjectRegistry,
    pending: HashSet<ObjectId>,
    trace: Vec<ObjectId>,
}

impl<'a> OrchestrationContext<'a> {
    pub fn new(
        class_registry: &'a ObjectClassRegistry,
        object_registry: &'a ObjectRegistry,
    ) -> Self {
        Self {
            class_registry,
            object_registry,
            pending: HashSet::new(),
            trace: Vec::new(),
        }
    }
}

impl Orchestration {
    pub fn rebuild_target<'c>(
        &mut self,
        target: &ObjectId,
        context: &mut OrchestrationContext<'c>,
    ) -> Result<(), OrchestrationError> {
        let object = context
            .object_registry
            .get(&target)
            .ok_or_else(|| OrchestrationError::ObjectNotFound(target.clone()))?;
        match object {
            super::registry::GetObject::Router(object) => {
                let class = context
                    .class_registry
                    .get_router(&object.class)
                    .ok_or(OrchestrationError::UndefinedClass(object.class.clone()))?;
                let class = class
                    .constructor
                    .construct(&object.config)
                    .map_err(|error| OrchestrationError::MakeError {
                        class_name: object.class.clone(),
                        node_id: target.clone(),
                        error,
                    })?;
                self.constructed_router.0.insert(target.clone(), class);
            }
            super::registry::GetObject::Layer(object) => {
                let class = context
                    .class_registry
                    .get_layer(&object.class)
                    .ok_or(OrchestrationError::UndefinedClass(object.class.clone()))?;
                let class = class
                    .constructor
                    .construct(&object.config)
                    .map_err(|error| OrchestrationError::MakeError {
                        class_name: object.class.clone(),
                        node_id: target.clone(),
                        error,
                    })?;
                self.constructed_layer.0.insert(target.clone(), class);
            }
            super::registry::GetObject::Service(object) => {
                let class = context
                    .class_registry
                    .get_service(&object.class)
                    .ok_or(OrchestrationError::UndefinedClass(object.class.clone()))?;
                let class = class
                    .constructor
                    .construct(&object.config)
                    .map_err(|error| OrchestrationError::MakeError {
                        class_name: object.class.clone(),
                        node_id: target.clone(),
                        error,
                    })?;
                self.constructed_service.0.insert(target.clone(), class);
            }
        }
        Ok(())
    }

    pub fn rebuild_all_target<'c>(
        &mut self,
        context: &mut OrchestrationContext<'c>,
    ) -> Result<(), OrchestrationError> {
        for id in context.object_registry.iter() {
            self.rebuild_target(id, context)?;
        }
        Ok(())
    }

    pub fn get_layers<'i>(
        &self,
        layer_ids: impl Iterator<Item = &'i ObjectId>,
    ) -> Result<Vec<SharedLayer>, OrchestrationError> {
        layer_ids
            .map(|layer_id| {
                self.constructed_layer
                    .0
                    .get(layer_id)
                    .cloned()
                    .ok_or(OrchestrationError::NotYetConstructed(layer_id.clone()))
            })
            .collect()
    }

    pub fn get_service<'i>(
        &self,
        service_id: &ObjectId,
    ) -> Result<SharedService, OrchestrationError> {
        self.constructed_service
            .0
            .get(service_id)
            .cloned()
            .ok_or(OrchestrationError::NotYetConstructed(service_id.clone()))
    }

    pub fn get_or_build_service<'c>(
        &mut self,
        id: &ObjectId,
        context: &mut OrchestrationContext<'c>,
    ) -> Result<SharedService, OrchestrationError> {
        if let Some(service) = self.built_services.get(id) {
            return Ok(service.clone());
        }
        if context.pending.contains(id) {
            return Err(OrchestrationError::LoopDetected {
                id: id.clone(),
                trace: context.trace.clone(),
            });
        }
        context.pending.insert(id.clone());
        context.trace.push(id.clone());
        let build_result = self.get_or_build_service(id, context);
        context.pending.remove(id);
        context.trace.pop();
        build_result
    }

    pub fn get_or_build_service_inner<'c>(
        &mut self,
        id: &ObjectId,
        context: &mut OrchestrationContext<'c>,
    ) -> Result<SharedService, OrchestrationError> {
        let object = context
            .object_registry
            .get(id)
            .ok_or_else(|| OrchestrationError::ObjectNotFound(id.clone()))?;
        let built_service = match object {
            super::registry::GetObject::Router(router_object) => {
                let layers: Vec<_> = self.get_layers(router_object.property.layers.iter())?;
                let mut services: BTreeMap<Route, SharedService> = BTreeMap::new();
                let router = self
                    .constructed_router
                    .0
                    .get(&router_object.id)
                    .ok_or_else(|| OrchestrationError::NotYetConstructed(router_object.id.clone()))?
                    .clone();
                for (route, id) in router_object.property.routes.iter() {
                    let service = self.get_or_build_service(id, context)?;
                    services.insert(route.clone(), service);
                }
                let mut inner_service =
                    SharedService::new(RouterService::dynamic_new(services, router));
                for layer in layers.into_iter() {
                    inner_service = Layer::layer(layer, inner_service.clone());
                }
                inner_service
            }
            super::registry::GetObject::Service(service_object) => {
                let layers: Vec<_> = self.get_layers(service_object.property.layers.iter())?;
                let mut inner_service = self
                    .constructed_service
                    .0
                    .get(&service_object.id)
                    .ok_or_else(|| {
                        OrchestrationError::NotYetConstructed(service_object.id.clone())
                    })?
                    .clone();
                for layer in layers.into_iter() {
                    inner_service = Layer::layer(layer, inner_service.clone());
                }
                inner_service
            }
            _ => return Err(OrchestrationError::BuildingServiceOnLayer(id.clone())),
        };
        self.built_services
            .insert(id.clone(), built_service.clone());
        Ok(built_service)
    }

    pub fn build_entries<'c>(
        &mut self,
        entry_points: impl Iterator<Item = &'c ObjectId>,
        context: &mut OrchestrationContext<'c>,
    ) -> Result<HashMap<ObjectId, SharedService>, OrchestrationError> {
        self.built_services.clear();
        let mut services = HashMap::new();
        for id in entry_points {
            let service = self.get_or_build_service(&id, context)?;
            services.insert(id.clone(), service);
        }
        Ok(services)
    }
}
