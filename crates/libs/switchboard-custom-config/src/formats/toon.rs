use crate::formats::TransferObject;

use super::Formats;

#[derive(Debug, Clone, Default)]
pub struct Toon;

impl Formats for Toon {
    type DecodeError = serde_toon::error::Error;
    type EncodeError = serde_toon::error::Error;

    fn format_name(&self) -> &'static str {
        "toon"
    }

    fn decode_bytes<T: TransferObject>(&self, bytes: bytes::Bytes) -> Result<T, Self::DecodeError> {
        let t = serde_toon::from_slice(&bytes)?;
        Ok(t)
    }

    fn encode_bytes<T: TransferObject>(
        &self,
        value: &T,
    ) -> Result<bytes::Bytes, Self::EncodeError> {
        let vec = serde_toon::to_string(value)?;
        Ok(bytes::Bytes::from(vec))
    }
}
