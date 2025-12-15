use crate::formats::PayloadObject;

use super::Formats;

#[derive(Debug, Clone, Default)]
pub struct Json;

impl Formats for Json {
    type DecodeError = serde_json::error::Error;
    type EncodeError = serde_json::error::Error;

    fn format_name(&self) -> &'static str {
        "json"
    }

    fn decode_bytes<T: PayloadObject>(&self, bytes: bytes::Bytes) -> Result<T, Self::DecodeError> {
        let t = serde_json::from_slice(&bytes)?;
        Ok(t)
    }

    fn encode_bytes<T: PayloadObject>(&self, value: &T) -> Result<bytes::Bytes, Self::EncodeError> {
        let vec = serde_json::to_vec(value)?;
        Ok(bytes::Bytes::from(vec))
    }
}
