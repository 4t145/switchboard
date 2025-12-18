use std::ops::{Deref, DerefMut};

use base64::prelude::*;

#[derive(Clone, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub struct Base64Bytes(pub Vec<u8>);

impl std::fmt::Debug for Base64Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Base64Bytes({} bytes)", self.0.len())
    }
}
impl Deref for Base64Bytes {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Base64Bytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl AsRef<[u8]> for Base64Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl serde::Serialize for Base64Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let b64 = BASE64_STANDARD.encode(&self.0);
        serializer.serialize_str(&b64)
    }
}

impl<'de> serde::Deserialize<'de> for Base64Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = BASE64_STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)?;
        Ok(Base64Bytes(bytes))
    }
}
