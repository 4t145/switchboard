
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
mod collection;
mod variant;
pub use collection::*;
mod option;
pub use option::*;
mod primitive;
pub use primitive::*;

use crate::variant::SerializeVariant;
pub mod macros;
macro_rules! for_primitives {
    ($($f:ident $T: ty;)*) => {
        $(
            fn $f<E>(self, v: $T) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SerdeValue::Primitive(primitive::SerdePrimitive::from(v)))
            }
        )*
    };
    (@ser $($f:ident $T: ty;)*) => {
        $(
            fn $f(self, v: $T) -> Result<Self::Ok, Self::Error>
            {
                Ok(SerdeValue::Primitive(primitive::SerdePrimitive::from(v)))
            }
        )*
    };
}

#[derive(Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[repr(u8)]
pub enum SerdeValue {
    #[default]
    Unit = 0,
    Primitive(primitive::SerdePrimitive) = 1,
    String(String) = 2,
    Bytes(Vec<u8>) = 3,
    Map(collection::SerdeMap) = 4,
    Sequence(collection::SerdeSequence) = 5,
    Option(option::SerdeOption) = 6,
}

impl std::fmt::Debug for SerdeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerdeValue::Unit => write!(f, "()"),
            SerdeValue::Primitive(p) => write!(f, "{:?}", p),
            SerdeValue::String(s) => write!(f, "\"{}\"", s),
            SerdeValue::Bytes(b) => write!(f, "Bytes({})", b.len()),
            SerdeValue::Map(m) => write!(f, "{:?}", m),
            SerdeValue::Sequence(c) => write!(f, "{:?}", c),
            SerdeValue::Option(o) => write!(f, "{:?}", o),
        }
    }
}

impl From<Vec<u8>> for SerdeValue {
    fn from(value: Vec<u8>) -> Self {
        SerdeValue::Bytes(value)
    }
}

impl From<&[u8]> for SerdeValue {
    fn from(value: &[u8]) -> Self {
        SerdeValue::Bytes(value.to_vec())
    }
}

impl From<()> for SerdeValue {
    fn from(_value: ()) -> Self {
        SerdeValue::Unit
    }
}

impl From<primitive::SerdePrimitive> for SerdeValue {
    fn from(value: primitive::SerdePrimitive) -> Self {
        SerdeValue::Primitive(value)
    }
}

impl From<String> for SerdeValue {
    fn from(value: String) -> Self {
        SerdeValue::String(value)
    }
}

impl From<&str> for SerdeValue {
    fn from(value: &str) -> Self {
        SerdeValue::String(value.to_string())
    }
}

impl From<collection::SerdeMap> for SerdeValue {
    fn from(value: collection::SerdeMap) -> Self {
        SerdeValue::Map(value)
    }
}

impl From<collection::SerdeSequence> for SerdeValue {
    fn from(value: collection::SerdeSequence) -> Self {
        SerdeValue::Sequence(value)
    }
}

impl From<option::SerdeOption> for SerdeValue {
    fn from(value: option::SerdeOption) -> Self {
        SerdeValue::Option(value)
    }
}


impl SerdeValue {
    pub fn serialize_from<T: Serialize>(value: &T) -> Result<Self, Error> {
        let serde_value = value.serialize(SerdeValueSerializer)?;
        Ok(serde_value)
    }
    pub fn deserialize_into<'de, T: Deserialize<'de>>(self) -> Result<T, Error> {
        T::deserialize(self)
    }
    /// map all Unit variants to the provided value recursively
    ///
    /// this is useful for some formats that do not support unit types directly
    pub fn replace_unit(self, value: crate::SerdeValue) -> Self {
        self.recursive_map(|x| matches!(x, SerdeValue::Unit), |_| value.clone())
    }

    pub(crate) fn recursive_map<F, M>(self, filter: F, map: M) -> Self
    where
        F: Fn(&crate::SerdeValue) -> bool,
        M: Fn(crate::SerdeValue) -> crate::SerdeValue,
    {
        if filter(&self) {
            return map(self);
        }
        match self {
            SerdeValue::Map(m) => {
                let mapped = m
                    .into_inner()
                    .into_iter()
                    .filter(|(k, _)| filter(k))
                    .map(|(k, v)| (k.clone(), map(v)))
                    .collect();
                SerdeValue::Map(collection::SerdeMap(mapped))
            }
            SerdeValue::Sequence(s) => {
                let mapped = s
                    .into_inner()
                    .into_iter()
                    .filter(|v| filter(v))
                    .map(&map)
                    .collect();
                SerdeValue::Sequence(collection::SerdeSequence(mapped))
            }
            SerdeValue::Option(o) => match o.0 {
                Some(value) => {
                    if filter(&value) {
                        SerdeValue::Option(option::SerdeOption(Some(Box::new(map(*value)))))
                    } else {
                        SerdeValue::Option(option::SerdeOption(None))
                    }
                }
                None => SerdeValue::Option(option::SerdeOption(None)),
            },
            other => other,
        }
    }
}

impl Serialize for SerdeValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SerdeValue::Unit => serializer.serialize_unit(),
            SerdeValue::Primitive(p) => p.serialize(serializer),
            SerdeValue::String(s) => serializer.serialize_str(s),
            SerdeValue::Bytes(b) => serializer.serialize_bytes(b),
            SerdeValue::Map(m) => m.serialize(serializer),
            SerdeValue::Sequence(c) => c.serialize(serializer),
            SerdeValue::Option(o) => o.serialize(serializer),
        }
    }
}

struct SerdeValueSerializer;

#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SerializeError: {}", self.0)
    }
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error(msg.to_string())
    }
}

impl Serializer for SerdeValueSerializer {
    type Ok = SerdeValue;
    type Error = Error;
    type SerializeSeq = collection::SerdeSequence;
    type SerializeTuple = collection::SerdeSequence;
    type SerializeTupleStruct = collection::SerdeSequence;
    type SerializeTupleVariant = SerializeVariant<collection::SerdeSequence>;
    type SerializeMap = collection::SerdeMap;
    type SerializeStruct = collection::SerdeMap;
    type SerializeStructVariant = SerializeVariant<collection::SerdeMap>;
    fn is_human_readable(&self) -> bool {
        false
    }

    for_primitives! {
        @ser
        serialize_i8 i8;
        serialize_i16 i16;
        serialize_i32 i32;
        serialize_i64 i64;
        serialize_u8 u8;
        serialize_u16 u16;
        serialize_u32 u32;
        serialize_u64 u64;
        serialize_f32 f32;
        serialize_f64 f64;
        serialize_bool bool;
        serialize_char char;
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        let value = option::SerdeOption(None);
        Ok(SerdeValue::Option(value))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(SerdeValueSerializer)?;
        let option_value = option::SerdeOption(Some(Box::new(value)));
        Ok(SerdeValue::Option(option_value))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::Unit)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::Unit)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(SerdeValue::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(SerdeValueSerializer)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(SerdeValueSerializer)?;
        Ok(SerdeValue::Map(SerdeMap(vec![(
            SerdeValue::String(variant.to_string().into()),
            v,
        )])))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(collection::SerdeSequence(Vec::with_capacity(
            len.unwrap_or(0),
        )))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(collection::SerdeSequence(Vec::with_capacity(len)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(collection::SerdeSequence(Vec::with_capacity(len)))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeVariant::new(
            variant,
            collection::SerdeSequence(Vec::with_capacity(len)),
        ))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(collection::SerdeMap(Vec::with_capacity(len.unwrap_or(0))))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(collection::SerdeMap(Vec::with_capacity(len)))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeVariant::new(
            variant,
            collection::SerdeMap(Vec::with_capacity(len)),
        ))
    }
}

struct SerdeAnyVisitor;

impl<'de> Visitor<'de> for SerdeAnyVisitor {
    type Value = SerdeValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a serde any value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeValue::Unit)
    }

    for_primitives! {
        visit_i8 i8;
        visit_i16 i16;
        visit_i32 i32;
        visit_i64 i64;
        visit_u8 u8;
        visit_u16 u16;
        visit_u32 u32;
        visit_u64 u64;
        visit_f32 f32;
        visit_f64 f64;
        visit_bool bool;
        visit_char char;
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeValue::String(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeValue::String(v))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeValue::String(v.to_string()))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let collection = collection::SerdeSequence::deserialize(
            serde::de::value::SeqAccessDeserializer::new(seq),
        )?;
        Ok(SerdeValue::Sequence(collection))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let collection =
            collection::SerdeMap::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
        Ok(SerdeValue::Map(collection))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeValue::Bytes(v.to_vec()))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeValue::Bytes(v))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdeValue::Bytes(v.to_vec()))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = option::SerdeOption::deserialize(deserializer)?;
        Ok(SerdeValue::Option(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let value = option::SerdeOption(None);
        Ok(SerdeValue::Option(value))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        SerdeValue::deserialize(deserializer)
    }
}
impl<'de> Deserialize<'de> for SerdeValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SerdeAnyVisitor)
    }
}

impl<'de> Deserializer<'de> for SerdeValue {
    type Error = Error;
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct seq newtype_struct tuple tuple_struct
        map identifier ignored_any struct
    }
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::Unit => visitor.visit_unit(),
            SerdeValue::Primitive(p) => p.deserialize_any(visitor),
            SerdeValue::String(s) => visitor.visit_string(s),
            SerdeValue::Bytes(b) => visitor.visit_byte_buf(b),
            SerdeValue::Map(m) => visitor.visit_map(m.access()),
            SerdeValue::Sequence(c) => visitor.visit_seq(c.access()),
            SerdeValue::Option(o) => match o.0 {
                Some(value) => visitor.visit_some(*value),
                None => visitor.visit_none(),
            },
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::Option(o) => match o.0 {
                Some(value) => visitor.visit_some(*value),
                None => visitor.visit_none(),
            },
            v => v.deserialize_any(visitor),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            SerdeValue::String(s) => {
                let de = serde::de::value::StrDeserializer::new(&s);
                visitor.visit_enum(de)
            }
            SerdeValue::Map(m) => {
                let mut map_access = m.access();
                visitor.visit_enum(serde::de::value::MapAccessDeserializer::new(
                    &mut map_access,
                ))
            }
            v => visitor.visit_newtype_struct(v),
        }
    }
}
