use crate::layer::dynamic::SharedLayer;

use super::{ObjectClassKindEnum, ObjectClassType};

impl ObjectClassType for SharedLayer {
    type Property = ();
    const KIND: ObjectClassKindEnum = ObjectClassKindEnum::Layer;
}
