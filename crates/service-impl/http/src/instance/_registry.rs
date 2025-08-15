use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    instance::{
        ClassKind,
        class::{Class, Constructor},
    },
    layer::{
        dynamic::SharedLayer,
        rewrite::{Rewrite, RewriteLayer},
    },
    router::{self, SharedRouter},
    service::{client::Client, dynamic::SharedService},
};

use super::{
    Instance, InstanceId,
    class::{ClassId, SbhClass},
};
#[derive(Clone)]
pub struct ClassWithConstructor<T> {
    pub constructor: Constructor<T>,
    pub class: Class,
}
pub trait ReflectKind {
    fn kind() -> ClassKind;
}

impl ReflectKind for SharedLayer {
    fn kind() -> ClassKind {
        ClassKind::Layer
    }
}

impl ReflectKind for SharedRouter {
    fn kind() -> ClassKind {
        ClassKind::Router
    }
}

impl ReflectKind for SharedService {
    fn kind() -> ClassKind {
        ClassKind::Service
    }
}

impl<T: ReflectKind> ClassWithConstructor<T> {
    pub fn from_sbh_class<C: SbhClass<Type = T>>(class: C) -> Self {
        let class_data = Class {
            id: class.id(),
            kind: C::Type::kind(),
            meta: class.meta(),
            config_schema: class.schema(),
        };
        Self {
            constructor: Constructor::new(move |config| {
                let config = serde_json::from_value(config.clone()).context("deserializing config")?;
                let constructed = class.construct(config).context("constructing class")?;
                anyhow::Ok(constructed)
            }),
            class: class_data,
        }
    }
}

impl<T> Serialize for ClassWithConstructor<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.class.serialize(serializer)
    }
}

impl<T> std::fmt::Debug for ClassWithConstructor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassWithConstructor")
            .field("class", &self.class)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct InstanceWithConstructed<T> {
    pub instance: Instance,
    pub constructed: T,
}

impl<T> Serialize for InstanceWithConstructed<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.instance.serialize(serializer)
    }
}

impl<T> InstanceWithConstructed<T> {
    pub fn new(instance: Instance, constructed: T) -> Self {
        Self {
            instance,
            constructed,
        }
    }
    pub fn class_id(&self) -> &ClassId {
        &self.instance.class
    }
    pub fn config(&self) -> &serde_json::Value {
        &self.instance.config
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClassRegistry {
    pub router: HashMap<ClassId, ClassWithConstructor<SharedRouter>>,
    pub layer: HashMap<ClassId, ClassWithConstructor<SharedLayer>>,
    pub service: HashMap<ClassId, ClassWithConstructor<SharedService>>,
}

pub enum GetObjectClass<'a> {
    Router(&'a ClassWithConstructor<SharedRouter>),
    Layer(&'a ClassWithConstructor<SharedLayer>),
    Service(&'a ClassWithConstructor<SharedService>),
}

impl ClassRegistry {
    pub fn globol() -> Arc<tokio::sync::RwLock<ClassRegistry>> {
        static INSTANCE: OnceLock<Arc<tokio::sync::RwLock<ClassRegistry>>> = OnceLock::new();
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
        self.register_router(router::PathMatch);
        self.register_router(router::Transparent);
    }
    pub fn register_service<C: SbhClass<Type = SharedService>>(&mut self, class: C) {
        let class_name = class.id();
        let class = ClassWithConstructor::from_sbh_class(class);
        self.service.insert(class_name, class);
    }
    pub fn register_layer<C: SbhClass<Type = SharedLayer>>(&mut self, class: C) {
        let class_name = class.id();
        let class = ClassWithConstructor::from_sbh_class(class);
        self.layer.insert(class_name, class);
    }
    pub fn register_router<C: SbhClass<Type = SharedRouter>>(&mut self, class: C) {
        let class_name = class.id();
        let class = ClassWithConstructor::from_sbh_class(class);
        self.router.insert(class_name, class);
    }
    pub fn unregister_service(&mut self, class_name: &ClassId) {
        self.service.remove(class_name);
    }
    pub fn unregister_layer(&mut self, class_name: &ClassId) {
        self.layer.remove(class_name);
    }
    pub fn unregister_router(&mut self, class_name: &ClassId) {
        self.router.remove(class_name);
    }
    pub fn get(&self, class_name: &ClassId) -> Option<GetObjectClass<'_>> {
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
    pub fn get_router(&self, class_name: &ClassId) -> Option<&ClassWithConstructor<SharedRouter>> {
        self.router.get(class_name)
    }
    pub fn get_layer(&self, class_name: &ClassId) -> Option<&ClassWithConstructor<SharedLayer>> {
        self.layer.get(class_name)
    }
    pub fn get_service(
        &self,
        class_name: &ClassId,
    ) -> Option<&ClassWithConstructor<SharedService>> {
        self.service.get(class_name)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct InstanceRegistry {
    pub router: HashMap<InstanceId, InstanceWithConstructed<SharedRouter>>,
    pub layer: HashMap<InstanceId, InstanceWithConstructed<SharedLayer>>,
    pub service: HashMap<InstanceId, InstanceWithConstructed<SharedService>>,
}

impl InstanceRegistry {
    pub fn new() -> Self {
        Self {
            router: HashMap::new(),
            layer: HashMap::new(),
            service: HashMap::new(),
        }
    }
    pub fn get(&self, id: &InstanceId) -> Option<GetInstance<'_>> {
        if let Some(object) = self.router.get(id) {
            return Some(GetInstance::Router(object));
        }
        if let Some(object) = self.layer.get(id) {
            return Some(GetInstance::Layer(object));
        }
        if let Some(object) = self.service.get(id) {
            return Some(GetInstance::Service(object));
        }
        None
    }
    pub fn iter(&self) -> impl Iterator<Item = &InstanceId> {
        self.router
            .keys()
            .chain(self.layer.keys())
            .chain(self.service.keys())
    }
}

pub enum GetInstance<'a> {
    Router(&'a InstanceWithConstructed<SharedRouter>),
    Layer(&'a InstanceWithConstructed<SharedLayer>),
    Service(&'a InstanceWithConstructed<SharedService>),
}
