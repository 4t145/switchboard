use serde::{Deserialize, Serialize, Serializer};
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[repr(u8)]
pub enum SerdePrimitive {
    Bool(bool) = 0,
    I8(i8) = 1,
    I16(i16) = 2,
    I32(i32) = 3,
    I64(i64) = 4,
    U8(u8) = 5,
    U16(u16) = 6,
    U32(u32) = 7,
    U64(u64) = 8,
    F32(f32) = 9,
    F64(f64) = 10,
    Char(char) = 11,
}

macro_rules! derive_from {
    ($($v: ident: $T: ty;)*) => {
        $(
            impl From<$T> for SerdePrimitive {
                fn from(value: $T) -> Self {
                    SerdePrimitive::$v(value)
                }
            }
        )*
    };
}

derive_from!(
    Bool: bool;
    I64: i64;
    I32: i32;
    I16: i16;
    I8: i8;
    U64: u64;
    U32: u32;
    U16: u16;
    U8: u8;
    F64: f64;
    F32: f32;
    Char: char;
);

impl Serialize for SerdePrimitive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SerdePrimitive::Bool(b) => serializer.serialize_bool(*b),
            SerdePrimitive::I64(i) => serializer.serialize_i64(*i),
            SerdePrimitive::I32(i) => serializer.serialize_i32(*i),
            SerdePrimitive::I16(i) => serializer.serialize_i16(*i),
            SerdePrimitive::I8(i) => serializer.serialize_i8(*i),
            SerdePrimitive::U64(u) => serializer.serialize_u64(*u),
            SerdePrimitive::U32(u) => serializer.serialize_u32(*u),
            SerdePrimitive::U16(u) => serializer.serialize_u16(*u),
            SerdePrimitive::U8(u) => serializer.serialize_u8(*u),
            SerdePrimitive::F64(f) => serializer.serialize_f64(*f),
            SerdePrimitive::F32(f) => serializer.serialize_f32(*f),
            SerdePrimitive::Char(c) => serializer.serialize_char(*c),
        }
    }
}

impl<'de> Deserialize<'de> for SerdePrimitive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(SerdePrimitiveVisitor)
    }
}

impl<'de> serde::de::Deserializer<'de> for SerdePrimitive {
    type Error = crate::Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            SerdePrimitive::Bool(b) => visitor.visit_bool(b),
            SerdePrimitive::I64(i) => visitor.visit_i64(i),
            SerdePrimitive::I32(i) => visitor.visit_i32(i),
            SerdePrimitive::I16(i) => visitor.visit_i16(i),
            SerdePrimitive::I8(i) => visitor.visit_i8(i),
            SerdePrimitive::U64(u) => visitor.visit_u64(u),
            SerdePrimitive::U32(u) => visitor.visit_u32(u),
            SerdePrimitive::U16(u) => visitor.visit_u16(u),
            SerdePrimitive::U8(u) => visitor.visit_u8(u),
            SerdePrimitive::F64(f) => visitor.visit_f64(f),
            SerdePrimitive::F32(f) => visitor.visit_f32(f),
            SerdePrimitive::Char(c) => visitor.visit_char(c),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }
}

pub(crate) struct SerdePrimitiveVisitor;

impl<'de> serde::de::Visitor<'de> for SerdePrimitiveVisitor {
    type Value = SerdePrimitive;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a primitive value")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::I64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::U64(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::F64(v))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::Bool(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::Char(v))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::I32(v))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::I16(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::I8(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::U32(v))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::U16(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::U8(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(SerdePrimitive::F32(v))
    }
}
