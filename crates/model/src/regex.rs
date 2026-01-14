use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug)]
pub struct SerdeRegex(pub regex::Regex);

impl Serialize for SerdeRegex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for SerdeRegex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let re = regex::Regex::new(&s).map_err(serde::de::Error::custom)?;
        Ok(SerdeRegex(re))
    }
}

impl Deref for SerdeRegex {
    type Target = regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerdeRegex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
