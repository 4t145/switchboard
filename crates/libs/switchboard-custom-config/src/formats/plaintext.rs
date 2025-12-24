use std::string::FromUtf8Error;

use switchboard_serde_value::SerdeValue;

use crate::formats::TransferObject;

use super::Formats;

#[derive(Debug, Clone, Default)]
pub struct Plaintext;

#[derive(Debug, thiserror::Error)]
pub enum PlaintextDecodeError {
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Serde value decode error: {0}")]
    DecodeError(#[from] switchboard_serde_value::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PlaintextEncodeError {
    #[error("Decode target type is not String")]
    InvalidType,

    #[error("Serde value encode error: {0}")]
    EncodeError(#[from] switchboard_serde_value::Error),
}

impl Formats for Plaintext {
    type EncodeError = PlaintextEncodeError;
    type DecodeError = PlaintextDecodeError;

    fn format_name(&self) -> &'static str {
        "plaintext"
    }

    fn decode_bytes<T: TransferObject>(&self, bytes: bytes::Bytes) -> Result<T, Self::DecodeError> {
        let s = String::from_utf8(bytes.to_vec())?;
        let value = SerdeValue::String(s);
        let value = value.deserialize_into()?;
        Ok(value)
    }

    fn encode_bytes<T: TransferObject>(
        &self,
        value: &T,
    ) -> Result<bytes::Bytes, Self::EncodeError> {
        let value: SerdeValue =
            SerdeValue::serialize_from(value).map_err(|_| PlaintextEncodeError::InvalidType)?;
        match value {
            SerdeValue::String(s) => Ok(bytes::Bytes::from(s.into_bytes())),
            _ => Err(PlaintextEncodeError::InvalidType),
        }
    }
}
