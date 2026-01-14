pub mod formats;

mod link;
use bytes::Bytes;
pub use link::*;
pub use switchboard_serde_value::{self, Error as SerdeValueError, SerdeValue};

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
// #[serde(untagged)]
// pub enum LinkOrValue<Value = SerdeValue> {
//     Link(Link),
//     Value(Value),
// }

// impl<V> LinkOrValue<V>
// where V: crate::formats::TransferObject
// {
//     pub fn is_link(&self) -> bool {
//         matches!(self, LinkOrValue::Link(_))
//     }
//     pub fn as_link(&self) -> Option<&Link> {
//         match self {
//             LinkOrValue::Link(link) => Some(link),
//             _ => None,
//         }
//     }
//     pub fn as_value(&self) -> Option<&V> {
//         match self {
//             LinkOrValue::Value(value) => Some(value),
//             _ => None,
//         }
//     }
// }

// impl<Value> LinkOrValue<Value>
// where Value: crate::formats::TransferObject
// {
//     pub async fn resolve<R: LinkResolver>(self, resolver: &R) -> Result<Value, Error> {
//         match self {
//             LinkOrValue::Link(link) => {
//                 let config = resolver.fetch::<Value>(&link).await?;
//                 Ok(config.value)
//             }
//             LinkOrValue::Value(value) => Ok(value),
//         }
//     }
// }
// impl Default for LinkOrValue {
//     fn default() -> Self {
//         LinkOrValue::Value(SerdeValue::default())
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigWithFormat<Value = SerdeValue> {
    pub value: Value,
    pub format: String,
}

impl Default for ConfigWithFormat {
    fn default() -> Self {
        Self {
            value: SerdeValue::default(),
            format: "json".to_string(),
        }
    }
}

impl<V> ConfigWithFormat<V>
where
    V: crate::formats::TransferObject,
{
    pub fn into_value(self) -> V {
        self.value
    }
    pub fn decode(format: &str, bytes: Bytes) -> Result<Self, Error> {
        let value = formats::decode_bytes(format, bytes)?;
        Ok(Self {
            value,
            format: format.to_string(),
        })
    }
    pub fn encode(&self) -> Result<Bytes, Error> {
        formats::encode_bytes(&self.format, &self.value)
    }
    pub fn new(format: impl Into<String>, value: V) -> Self {
        Self {
            value,
            format: format.into(),
        }
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
    UnknownFormat { format: String },
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
