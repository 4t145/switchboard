use std::string::FromUtf8Error;

use crate::formats::TransferObject;

use super::Formats;

#[derive(Debug, Clone, Default)]
pub struct Plaintext;

#[derive(Debug, thiserror::Error)]
pub enum PlaintextDecodeError {
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Decode target type is not String")]
    InvalidType,
}

#[derive(Debug, thiserror::Error)]
pub enum PlaintextEncodeError {
    #[error("Decode target type is not String")]
    InvalidType,
}

impl Formats for Plaintext {
    type EncodeError = PlaintextEncodeError;
    type DecodeError = PlaintextDecodeError;

    fn format_name(&self) -> &'static str {
        "plaintext"
    }

    fn decode_bytes<T: TransferObject>(&self, bytes: bytes::Bytes) -> Result<T, Self::DecodeError> {
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<String>() {
            let s = String::from_utf8(bytes.to_vec())?;
            // SAFETY: We just checked that T is String
            let t = unsafe { std::mem::transmute_copy::<String, T>(&s) };
            std::mem::forget(s);
            Ok(t)
        } else {
            Err(PlaintextDecodeError::InvalidType)
        }
    }

    fn encode_bytes<T: TransferObject>(&self, value: &T) -> Result<bytes::Bytes, Self::EncodeError> {
        let vec = if std::any::TypeId::of::<T>() == std::any::TypeId::of::<String>() {
            let s: &String = unsafe { &*(value as *const T as *const String) };
            s.as_bytes().to_vec()
        } else {
            return Err(PlaintextEncodeError::InvalidType);
        };
        Ok(bytes::Bytes::from(vec))
    }
}
