use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    object::ObjectId,
    router::{Route, SharedRouter},
};

use super::{ObjectClassKindEnum, ObjectClassType};

impl ObjectClassType for SharedRouter {
    type Property = RouterProperty;
    const KIND: ObjectClassKindEnum = ObjectClassKindEnum::Router;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouterProperty {
    pub routes: BTreeMap<Route, ObjectId>,
    pub layers: Vec<ObjectId>,
}
