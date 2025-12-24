use crate::formats::TransferObject;

use super::Formats;
#[derive(Debug, Clone, Default)]
pub struct Toml;

impl Formats for Toml {
    type DecodeError = toml::de::Error;
    type EncodeError = toml::ser::Error;

    fn format_name(&self) -> &'static str {
        "toml"
    }

    fn decode_bytes<T: TransferObject>(&self, bytes: bytes::Bytes) -> Result<T, Self::DecodeError> {
        let t = toml::from_slice(&bytes)?;
        Ok(t)
    }

    fn encode_bytes<T: TransferObject>(&self, value: &T) -> Result<bytes::Bytes, Self::EncodeError> {
        let vec = toml::to_string(value)?;
        Ok(bytes::Bytes::from(vec))
    }
}
