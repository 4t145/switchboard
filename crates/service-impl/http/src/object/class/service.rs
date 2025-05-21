use serde::{Deserialize, Serialize};

use crate::{object::ObjectId, service::dynamic::SharedService};

use super::{ObjectClassKindEnum, ObjectClassType};

impl ObjectClassType for SharedService {
    type Property = ServiceProperty;
    const KIND: ObjectClassKindEnum = ObjectClassKindEnum::Service;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceProperty {
    pub layers: Vec<ObjectId>,
}
