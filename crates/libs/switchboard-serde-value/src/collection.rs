use std::ops::{Deref, DerefMut};

use serde::{
    Deserialize, Serialize,
    ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeTuple, SerializeTupleStruct},
};

use crate::SerdeValue;

#[derive(Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SerdeSequence(pub Vec<crate::SerdeValue>);

impl std::fmt::Debug for SerdeSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<T> From<Vec<T>> for SerdeSequence
where
    T: Into<crate::SerdeValue>,
{
    fn from(value: Vec<T>) -> Self {
        SerdeSequence(value.into_iter().map(Into::into).collect())
    }
}

impl<const N: usize, T> From<[T; N]> for SerdeSequence
where
    T: Into<crate::SerdeValue>,
{
    fn from(value: [T; N]) -> Self {
        SerdeSequence(value.into_iter().map(Into::into).collect())
    }
}

impl Deref for SerdeSequence {
    type Target = Vec<crate::SerdeValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerdeSequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SerdeSequence {
    pub(crate) fn access(self) -> SerdeSequenceAccess {
        SerdeSequenceAccess(self.0.into_iter())
    }
    pub fn into_inner(self) -> Vec<crate::SerdeValue> {
        self.0
    }
}
impl serde::Serialize for SerdeSequence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq_serializer = serializer.serialize_seq(Some(self.0.len()))?;
        for element in &self.0 {
            seq_serializer.serialize_element(element)?;
        }
        seq_serializer.end()
    }
}
impl SerializeSeq for SerdeSequence {
    type Ok = SerdeValue;
    type Error = crate::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(crate::SerdeValueSerializer)?;
        self.0.push(value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::Sequence(self))
    }
}

impl SerializeTuple for SerdeSequence {
    type Ok = crate::SerdeValue;
    type Error = crate::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(crate::SerdeValueSerializer)?;
        self.0.push(value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::Sequence(self))
    }
}

impl SerializeTupleStruct for SerdeSequence {
    type Ok = crate::SerdeValue;
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(crate::SerdeValueSerializer)?;
        self.0.push(value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::Sequence(self))
    }
}

pub(crate) struct SerdeSequenceAccess(std::vec::IntoIter<crate::SerdeValue>);
impl<'de> serde::de::SeqAccess<'de> for SerdeSequenceAccess {
    type Error = crate::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if let Some(value) = self.0.next() {
            let de = value;
            let v = seed.deserialize(de)?;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
}

impl<'de> Deserialize<'de> for SerdeSequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SerdeSequenceVisitor;
        impl<'de> serde::de::Visitor<'de> for SerdeSequenceVisitor {
            type Value = SerdeSequence;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("collection")
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut elements = Vec::new();
                let mut seq = seq;
                while let Some(value) = seq.next_element::<crate::SerdeValue>()? {
                    elements.push(value);
                }
                Ok(SerdeSequence(elements))
            }
        }

        deserializer.deserialize_any(SerdeSequenceVisitor)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SerdeMap(pub Vec<(crate::SerdeValue, crate::SerdeValue)>);

impl SerdeMap {
    pub fn new() -> Self {
        SerdeMap(Vec::new())
    }
    pub fn add_entry(&mut self, key: crate::SerdeValue, value: crate::SerdeValue) {
        self.0.push((key, value));
    }
}

impl std::fmt::Debug for SerdeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

impl Deref for SerdeMap {
    type Target = Vec<(crate::SerdeValue, crate::SerdeValue)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerdeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SerdeMap {
    pub(crate) fn access(self) -> SerdeMapAccess {
        SerdeMapAccess {
            rest_entries: self.0.into_iter(),
            value: None,
        }
    }
    pub fn into_inner(self) -> Vec<(crate::SerdeValue, crate::SerdeValue)> {
        self.0
    }
}

pub(crate) struct SerdeMapAccess {
    rest_entries: std::vec::IntoIter<(crate::SerdeValue, crate::SerdeValue)>,
    value: Option<crate::SerdeValue>,
}

impl SerializeMap for SerdeMap {
    type Ok = SerdeValue;
    type Error = crate::Error;

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let key = key.serialize(crate::SerdeValueSerializer)?;
        let value = value.serialize(crate::SerdeValueSerializer)?;
        self.0.push((key, value));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::Map(self))
    }

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!("should delegate to serialize_entry")
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!("should delegate to serialize_entry")
    }
}

impl SerializeStruct for SerdeMap {
    type Ok = SerdeValue;
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, key: &str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(crate::SerdeValueSerializer)?;
        let value = value.serialize(crate::SerdeValueSerializer)?;
        self.0.push((key, value));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::Map(self))
    }
}

impl Serialize for SerdeMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map_serializer = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map_serializer.serialize_entry(key, value)?;
        }
        map_serializer.end()
    }
}

impl<'de> serde::de::MapAccess<'de> for SerdeMapAccess {
    type Error = crate::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.rest_entries.next() {
            self.value = Some(value);
            let de = key;
            let k = seed.deserialize(de)?;
            Ok(Some(k))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let de = self.value.take().expect("value must be present");
        let v = seed.deserialize(de)?;
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for SerdeMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SerdeMapVisitor;

        impl<'de> serde::de::Visitor<'de> for SerdeMapVisitor {
            type Value = SerdeMap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("map")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some((key, value)) =
                    map.next_entry::<crate::SerdeValue, crate::SerdeValue>()?
                {
                    values.push((key, value));
                }
                Ok(SerdeMap(values))
            }
        }

        deserializer.deserialize_map(SerdeMapVisitor)
    }
}
