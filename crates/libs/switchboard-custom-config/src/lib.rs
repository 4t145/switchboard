pub mod formats;
#[cfg(feature = "fs")]
pub mod fs;
mod link;
use bytes::Bytes;
pub use link::{Link, LinkResolver};
pub use switchboard_serde_value::{SerdeValue, self, Error as SerdeValueError};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
#[serde(untagged)]
pub enum LinkOrValue {
    Link(Link),
    Value(SerdeValue),
}

impl Default for LinkOrValue {
    fn default() -> Self {
        LinkOrValue::Value(SerdeValue::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomConfig {
    pub value: SerdeValue,
    pub format: String,
}

impl Default for CustomConfig {
    fn default() -> Self {
        Self {
            value: SerdeValue::default(),
            format: "json".to_string(),
        }
    }
}

impl CustomConfig {
    pub fn into_value(self) -> SerdeValue {
        self.value
    }
    pub fn decode(format: &str, bytes: Bytes) -> Result<Self, Error> {
        let value = formats::decode_bytes(format, bytes)?;
        Ok(Self { value, format: format.to_string() })
    }
    pub fn encode(&self) -> Result<Bytes, Error> {
        formats::encode_bytes(&self.format, &self.value)
    }
    pub fn new(format: impl Into<String>, value: SerdeValue) -> Self {
        Self { value, format: format.into() }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("link resolve error: {source}")]
    ResolveError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        link: Link,
    },
    #[error("unsupported format: {format}")]
    UnknownFormat {
        format: String,
    },
    #[error(transparent)]
    DecodeError(#[from] formats::DecodeError),
    #[error(transparent)]
    EncodeError(#[from] formats::EncodeError),
}

impl Error {
    pub fn resolve_error<E>(source: E, link: Link) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::ResolveError {
            source: source.into(),
            link,
        }
    }
}