    use crate::formats::TransferObject;

use super::Formats;

#[derive(Debug, Clone, Default)]
pub struct Bincode;

impl Formats for Bincode {
    type EncodeError = bincode::error::EncodeError;
    type DecodeError = bincode::error::DecodeError;

    fn format_name(&self) -> &'static str {
        "bincode"
    }

    fn decode_bytes<T: TransferObject>(&self, bytes: bytes::Bytes) -> Result<T, Self::DecodeError> {
        let (t, _) = bincode::decode_from_slice(&bytes, bincode::config::standard())?;
        Ok(t)
    }

    fn encode_bytes<T: TransferObject>(&self, value: &T) -> Result<bytes::Bytes, Self::EncodeError> {
        let vec = bincode::encode_to_vec(value, bincode::config::standard())?;
        Ok(bytes::Bytes::from(vec))
    }
}
