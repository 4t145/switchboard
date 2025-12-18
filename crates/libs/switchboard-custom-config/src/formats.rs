use bytes::Bytes;

pub mod bincode;
pub mod json;
pub mod plaintext;
pub mod toml;
pub mod toon;
pub trait Formats {
    type EncodeError: std::error::Error + Send + Sync + 'static;
    type DecodeError: std::error::Error + Send + Sync + 'static;
    fn format_name(&self) -> &'static str;
    fn decode_bytes<T: PayloadObject>(&self, bytes: Bytes) -> Result<T, Self::DecodeError>;
    fn encode_bytes<T: PayloadObject>(&self, value: &T) -> Result<Bytes, Self::EncodeError>;
}

#[derive(Debug, thiserror::Error)]
#[error("Decode error in format {format}: {source}")]
pub struct DecodeError {
    #[source]
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub format: String,
}

impl DecodeError {
    pub fn new<E>(source: E, format: &str) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            source: Box::new(source),
            format: format.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Encode error in format {format}: {source}")]
pub struct EncodeError {
    #[source]
    pub source: Box<dyn std::error::Error + Send + Sync>,
    pub format: String,
}

impl EncodeError {
    pub fn new<E>(source: E, format: &str) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            source: Box::new(source),
            format: format.to_string(),
        }
    }
}

pub trait PayloadObject:
    serde::Serialize
    + serde::de::DeserializeOwned
    + ::bincode::Encode
    + ::bincode::Decode<()>
    + std::any::Any
{
}

impl<T> PayloadObject for T where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + ::bincode::Encode
        + ::bincode::Decode<()>
        + std::any::Any
{
}

pub fn decode_bytes<T: PayloadObject>(format: &[u8], bytes: Bytes) -> Result<T, crate::Error> {
    let value = match format {
        b"bincode" => {
            let formatter = bincode::Bincode::default();
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        b"json" => {
            let formatter = json::Json::default();
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        b"toml" => {
            let formatter = toml::Toml::default();
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        b"toon" => {
            let formatter = toon::Toon::default();
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        b"plaintext" => {
            let formatter = plaintext::Plaintext::default();
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        _ => {
            return Err(crate::Error::UnknownFormat {
                format: format.to_vec(),
            });
        }
    };
    Ok(value)
}

pub fn encode_bytes<T: PayloadObject>(format: &[u8], value: &T) -> Result<Bytes, crate::Error> {
    let bytes = match format {
        b"bincode" => {
            let formatter = bincode::Bincode::default();
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        b"json" => {
            let formatter = json::Json::default();
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        b"toml" => {
            let formatter = toml::Toml::default();
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        b"toon" => {
            let formatter = toon::Toon::default();
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        b"plaintext" => {
            let formatter = plaintext::Plaintext::default();
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        _ => {
            return Err(crate::Error::UnknownFormat {
                format: format.to_vec(),
            });
        }
    };
    Ok(bytes)
}

