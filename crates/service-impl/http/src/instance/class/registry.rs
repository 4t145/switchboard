use std::collections::HashMap;

use anyhow::Context as _;

use crate::instance::{
    InstanceValue,
    class::{Class, ClassData, ClassId, Constructor},
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

pub enum ClassRegistryError {
    ConstructError(anyhow::Error),
    ClassNotFound { id: ClassId },
}

impl ClassRegistry {
    pub fn construct(
        &self,
        class_id: ClassId,
        config: serde_json::Value,
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
            config_schema: class.schema(),
        };
        self.class_data.insert(
            class_id,
            ClassDataWithConstructor {
                data: class_data,
                constructor: Constructor::new(move |config| {
                    let config =
                        serde_json::from_value(config.clone()).context("deserializing config")?;
                    class.construct(config).context("constructing class")
                }),
            },
        );
    }
}
