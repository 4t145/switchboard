use serde::ser::{SerializeStructVariant, SerializeTupleVariant};

pub struct SerializeVariant<S> {
    pub variant: &'static str,
    pub inner: S,
}

impl<S> SerializeVariant<S> {
    pub fn new(variant: &'static str, inner: S) -> Self {
        Self { variant, inner }
    }
}

impl SerializeStructVariant for SerializeVariant<crate::collection::SerdeMap> {
    type Ok = crate::SerdeValue;
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let value = value.serialize(crate::SerdeValueSerializer)?;
        self.inner
            .0
            .push((crate::SerdeValue::String(key.to_string()), value));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = crate::collection::SerdeMap::new();
        map.add_entry(
            crate::SerdeValue::String(self.variant.to_string()),
            crate::SerdeValue::Map(self.inner),
        );
        Ok(crate::SerdeValue::Map(map))
    }
}

impl SerializeTupleVariant for SerializeVariant<crate::collection::SerdeSequence> {
    type Ok = crate::SerdeValue;
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let value = value.serialize(crate::SerdeValueSerializer)?;
        self.inner.0.push(value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = crate::collection::SerdeMap::new();
        map.add_entry(
            crate::SerdeValue::String(self.variant.to_string()),
            crate::SerdeValue::Sequence(self.inner),
        );
        Ok(crate::SerdeValue::Map(map))
    }
}
