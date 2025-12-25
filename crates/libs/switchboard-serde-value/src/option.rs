use std::ops::{Deref, DerefMut};

use serde::Deserialize;

use crate::SerdeValue;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SerdeOption(pub Option<Box<SerdeValue>>);

impl From<Option<SerdeValue>> for SerdeOption {
    fn from(value: Option<SerdeValue>) -> Self {
        SerdeOption(value.map(Box::new))
    }
}

impl From<SerdeOption> for Option<SerdeValue> {
    fn from(value: SerdeOption) -> Self {
        value.0.map(|b| *b)
    }
}

impl From<Option<Box<SerdeValue>>> for SerdeOption {
    fn from(value: Option<Box<SerdeValue>>) -> Self {
        SerdeOption(value)
    }
}

impl SerdeOption {
    pub fn into_inner(self) -> Option<Box<SerdeValue>> {
        self.0
    }
    pub fn into_deref(self) -> Option<SerdeValue> {
        self.0.map(|b| *b)
    }
    pub fn as_ref(&self) -> Option<&SerdeValue> {
        self.0.as_deref()
    }
}

impl Deref for SerdeOption {
    type Target = Option<Box<SerdeValue>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerdeOption {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl serde::Serialize for SerdeOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Some(value) => serializer.serialize_some(value),
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for SerdeOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SerdeOptionVisitor;
        impl<'de> serde::de::Visitor<'de> for SerdeOptionVisitor {
            type Value = SerdeOption;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("option")
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = crate::SerdeValue::deserialize(deserializer)?;
                Ok(SerdeOption(Some(Box::new(value))))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SerdeOption(None))
            }
        }

        deserializer.deserialize_option(SerdeOptionVisitor)
    }
}
