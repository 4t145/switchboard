pub mod class;
pub mod orchestration;
pub mod registry;

use std::{ops::Deref, sync::Arc};

use class::{ObjectClassName, ObjectClassType};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct ObjectId(Arc<str>);

impl Deref for ObjectId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl std::fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjectId({})", self.0)
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

impl ObjectId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
    pub fn random() -> Self {
        Self(Arc::from(uuid::Uuid::new_v4().to_string()))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Object<C: ObjectClassType> {
    pub id: ObjectId,
    pub class: ObjectClassName,
    #[serde(flatten)]
    pub property: C::Property,
    pub config: String,
}

impl<C: ObjectClassType> std::fmt::Debug for Object<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("id", &self.id)
            .field("class", &self.class)
            .field("property", &self.property)
            .field("config", &self.config)
            .finish()
    }
}
