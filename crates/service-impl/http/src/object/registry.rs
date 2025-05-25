use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use serde::{Deserialize, Serialize};

use crate::{
    layer::{
        dynamic::SharedLayer,
        rewrite::{Rewrite, RewriteLayer},
    },
    router::{self, SharedRouter},
    service::{client::Client, dynamic::SharedService},
};

use super::{
    Object, ObjectId,
    class::{ObjectClass, ObjectClassName, ObjectClassType, SbhClass},
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
        INSTANCE
            .get_or_init(|| {
                let mut instance = Self::default();
                instance.prelude();
                Arc::new(tokio::sync::RwLock::new(instance))
            })
            .clone()
    }
    pub fn prelude(&mut self) {
        // service
        self.register_service(Client);

        // layer
        self.register_layer(Rewrite);

        // router
        self.register_router(router::Host);
        self.register_router(router::Path);
        self.register_router(router::Transparent);
    }
    pub fn register_service<C: SbhClass<Type = SharedService>>(&mut self, class: C) {
        let class_name = class.name();
        let class = ObjectClass::from_sbh_class(class);
        self.service.insert(class_name, class);
    }
    pub fn register_layer<C: SbhClass<Type = SharedLayer>>(&mut self, class: C) {
        let class_name = class.name();
        let class = ObjectClass::from_sbh_class(class);
        self.layer.insert(class_name, class);
    }
    pub fn register_router<C: SbhClass<Type = SharedRouter>>(&mut self, class: C) {
        let class_name = class.name();
        let class = ObjectClass::from_sbh_class(class);
        self.router.insert(class_name, class);
    }
    pub fn unregister_service(&mut self, class_name: &ObjectClassName) {
        self.service.remove(class_name);
    }
    pub fn unregister_layer(&mut self, class_name: &ObjectClassName) {
        self.layer.remove(class_name);
    }
    pub fn unregister_router(&mut self, class_name: &ObjectClassName) {
        self.router.remove(class_name);
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
    pub fn new() -> Self {
        Self {
            router: HashMap::new(),
            layer: HashMap::new(),
            service: HashMap::new(),
        }
    }
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
