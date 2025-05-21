use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use serde::{Deserialize, Serialize};

use crate::{layer::dynamic::SharedLayer, router::SharedRouter, service::dynamic::SharedService};

use super::{
    Object, ObjectId,
    class::{ObjectClass, ObjectClassName},
};
#[derive(Debug, Clone, Default)]
pub struct ObjectClassRegistry {
    pub router: HashMap<ObjectClassName, ObjectClass<SharedRouter>>,
    pub layer: HashMap<ObjectClassName, ObjectClass<SharedLayer>>,
    pub service: HashMap<ObjectClassName, ObjectClass<SharedService>>,
}

pub enum GetObjectClass<'a> {
    Router(&'a ObjectClass<SharedRouter>),
    Layer(&'a ObjectClass<SharedLayer>),
    Service(&'a ObjectClass<SharedService>),
}

impl ObjectClassRegistry {
    pub fn globol() -> Arc<tokio::sync::RwLock<ObjectClassRegistry>> {
        static INSTANCE: OnceLock<Arc<tokio::sync::RwLock<ObjectClassRegistry>>> = OnceLock::new();
        INSTANCE.get_or_init(Default::default).clone()
    }
    pub fn get(&self, class_name: &ObjectClassName) -> Option<GetObjectClass<'_>> {
        if let Some(class) = self.router.get(class_name) {
            return Some(GetObjectClass::Router(class));
        }
        if let Some(class) = self.layer.get(class_name) {
            return Some(GetObjectClass::Layer(class));
        }
        if let Some(class) = self.service.get(class_name) {
            return Some(GetObjectClass::Service(class));
        }
        None
    }
    pub fn get_router(&self, class_name: &ObjectClassName) -> Option<&ObjectClass<SharedRouter>> {
        self.router.get(class_name)
    }
    pub fn get_layer(&self, class_name: &ObjectClassName) -> Option<&ObjectClass<SharedLayer>> {
        self.layer.get(class_name)
    }
    pub fn get_service(&self, class_name: &ObjectClassName) -> Option<&ObjectClass<SharedService>> {
        self.service.get(class_name)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectRegistry {
    pub router: HashMap<ObjectId, Object<SharedRouter>>,
    pub layer: HashMap<ObjectId, Object<SharedLayer>>,
    pub service: HashMap<ObjectId, Object<SharedService>>,
}

impl ObjectRegistry {
    pub fn get(&self, id: &ObjectId) -> Option<GetObject<'_>> {
        if let Some(object) = self.router.get(id) {
            return Some(GetObject::Router(object));
        }
        if let Some(object) = self.layer.get(id) {
            return Some(GetObject::Layer(object));
        }
        if let Some(object) = self.service.get(id) {
            return Some(GetObject::Service(object));
        }
        None
    }
    pub fn iter(&self) -> impl Iterator<Item = &ObjectId> {
        self.router
            .keys()
            .chain(self.layer.keys())
            .chain(self.service.keys())
    }
}

pub enum GetObject<'a> {
    Router(&'a Object<SharedRouter>),
    Layer(&'a Object<SharedLayer>),
    Service(&'a Object<SharedService>),
}
