use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use anyhow::Context as _;
use switchboard_service::CustomConfig;
use tokio::sync::RwLock;

use crate::{
    flow::{
        filter::{
            AsFilterClass, FilterClass, 
            timeout::Timeout, 
            url_rewrite::UrlRewriteFilterClass,
            request_mirror::RequestMirrorFilterClass,
            request_header_modify::RequestHeaderModifyFilterClass,
            response_header_modify::ResponseHeaderModifyFilterClass,
        },
        node::{AsNodeClass, NodeClass},
        // router::{host_match::HostMatch, path_match::PathMatch},
        service::client::Client,
    },
    instance::{
        InstanceValue,
        class::{Class, ClassData, ClassId, Constructor},
    },
};

#[derive(Clone)]
pub struct ClassDataWithConstructor {
    pub data: ClassData,
    pub constructor: Constructor,
}

impl std::fmt::Debug for ClassDataWithConstructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassDataWithConstructor")
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClassRegistry {
    pub class_data: HashMap<ClassId, ClassDataWithConstructor>,
}
#[derive(Debug, thiserror::Error)]
pub enum ClassRegistryError {
    #[error("Construct Error")]
    ConstructError(#[from] anyhow::Error),
    #[error("Class `{id}` not found")]
    ClassNotFound { id: ClassId },
}

impl ClassRegistry {
    pub fn const_new() -> Self {
        Self {
            class_data: HashMap::new(),
        }
    }
    pub fn construct(
        &self,
        class_id: ClassId,
        config: CustomConfig,
    ) -> Result<InstanceValue, ClassRegistryError> {
        let class_data = self
            .class_data
            .get(&class_id)
            .ok_or_else(|| ClassRegistryError::ClassNotFound { id: class_id })?;
        class_data
            .constructor
            .construct(&config)
            .map_err(ClassRegistryError::ConstructError)
    }
    pub fn register<C: Class>(&mut self, class: C) {
        let class_id = class.id();
        let class_data = ClassData {
            id: class_id.clone(),
            meta: class.meta(),
            instance_type: class.instance_type(),
            // config_schema: class.schema(),
        };
        self.class_data.insert(
            class_id,
            ClassDataWithConstructor {
                data: class_data,
                constructor: Constructor::new(move |config| {
                    let config = config.decode().context("deserializing config")?;
                    class.construct(config).context("constructing class")
                }),
            },
        );
    }
    pub fn register_node<N: NodeClass>(&mut self, class: N) {
        self.register(AsNodeClass(class));
    }
    pub fn register_filter<F: FilterClass>(&mut self, class: F) {
        self.register(AsFilterClass(class));
    }
    pub fn register_prelude(&mut self) {
        // self.register_node(PathMatch);
        // self.register_node(HostMatch);
        self.register_node(Client);
        self.register_filter(UrlRewriteFilterClass);
        self.register_filter(RequestMirrorFilterClass);
        self.register_filter(RequestHeaderModifyFilterClass);
        self.register_filter(ResponseHeaderModifyFilterClass);
        self.register_filter(Timeout);
    }
    pub fn global() -> Arc<RwLock<Self>> {
        GLOBAL_CLASS_REGISTRY
            .get_or_init(|| {
                let mut registry = Self::default();
                registry.register_prelude();
                Arc::new(RwLock::new(registry))
            })
            .clone()
    }
}

static GLOBAL_CLASS_REGISTRY: OnceLock<Arc<RwLock<ClassRegistry>>> = OnceLock::new();
