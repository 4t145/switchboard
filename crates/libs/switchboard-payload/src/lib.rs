pub mod formats;
#[cfg(feature = "fs")]
pub mod fs;
use base64::prelude::*;
use bytes::{Buf as _, BufMut};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BytesPayload(pub bytes::Bytes);

impl Default for BytesPayload {
    fn default() -> Self {
        BytesPayload::new("plaintext", "")
    }
}

impl serde::Serialize for BytesPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let encoded = BASE64_STANDARD.encode(&self.0);
            serializer.serialize_str(&encoded)
        } else {
            bytes::Bytes::serialize(&self.0, serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for BytesPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let decoded = BASE64_STANDARD
                .decode(s.as_bytes())
                .map_err(serde::de::Error::custom)?;
            Ok(BytesPayload(bytes::Bytes::from(decoded)))
        } else {
            let bytes = <bytes::Bytes>::deserialize(deserializer)?;
            Ok(BytesPayload(bytes))
        }
    }
}

impl bincode::Encode for BytesPayload {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.encode(encoder)
    }
}

impl<C> bincode::Decode<C> for BytesPayload {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let bytes = Vec::<u8>::decode(decoder)?;
        Ok(BytesPayload(bytes.into()))
    }
}

impl<'de, C> bincode::BorrowDecode<'de, C> for BytesPayload {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let bytes = Vec::<u8>::borrow_decode(decoder)?;
        Ok(BytesPayload(bytes.into()))
    }
}

impl BytesPayload {
    pub fn new(format: impl AsRef<[u8]>, bytes: impl AsRef<[u8]>) -> Self {
        let format_bytes = format.as_ref();
        let body_bytes = bytes.as_ref();
        let mut buf = bytes::BytesMut::with_capacity(4 + 1 + format_bytes.len() + body_bytes.len());
        buf.put_u32(body_bytes.len() as u32);
        buf.put_u8(format_bytes.len() as u8);
        buf.put_slice(format_bytes);
        buf.put_slice(body_bytes);
        BytesPayload(buf.freeze())
    }
    pub fn decode<T: formats::PayloadObject>(&self) -> Result<T, Error> {
        let mut buf = self.0.clone();
        let size = buf.get_u32();
        // check rest of size
        if size as usize != buf.remaining() {
            return Err(Error::InvalidPayloadSize {
                expected: size as usize,
                actual: buf.remaining(),
            });
        }
        let format_size = buf.get_u8();
        // check format size
        if format_size as usize > buf.remaining() {
            return Err(Error::InvalidFormatSize {
                at_least: format_size as usize,
                remain: buf.remaining(),
            });
        }
        let format = buf.split_to(format_size as usize);
        let body = buf;
        // decode body
        let value = formats::decode_bytes::<T>(&format, body)?;
        Ok(value)
    }
    pub fn encode<T: formats::PayloadObject>(
        format: impl AsRef<[u8]>,
        value: &T,
    ) -> Result<Self, Error> {
        let body_bytes = formats::encode_bytes(format.as_ref(), value)?;
        Ok(BytesPayload::new(format.as_ref(), body_bytes))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid payload size: expected {expected}, got {actual}")]
    InvalidPayloadSize { expected: usize, actual: usize },
    #[error("Invalid format size: expected at least {at_least}, got {remain}")]
    InvalidFormatSize { at_least: usize, remain: usize },
    #[error("Unknown format: {format}", format = String::from_utf8_lossy(&format))]
    UnknownFormat { format: Vec<u8> },
    #[error(transparent)]
    DecodeError(#[from] formats::DecodeError),
    #[error(transparent)]
    EncodeError(#[from] formats::EncodeError),
}
