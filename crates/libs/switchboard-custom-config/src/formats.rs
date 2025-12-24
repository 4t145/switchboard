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
    fn decode_bytes<T: TransferObject>(&self, bytes: Bytes) -> Result<T, Self::DecodeError>;
    fn encode_bytes<T: TransferObject>(&self, value: &T) -> Result<Bytes, Self::EncodeError>;
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

pub trait TransferObject:
    serde::Serialize
    + serde::de::DeserializeOwned
    + ::bincode::Encode
    + ::bincode::Decode<()>
    + std::any::Any
    + Send
    + Sync
{
}

impl<T> TransferObject for T where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + ::bincode::Encode
        + ::bincode::Decode<()>
        + std::any::Any
        + Send
        + Sync
{
}

pub fn decode_bytes<T: TransferObject>(
    format: impl AsRef<str>,
    bytes: Bytes,
) -> Result<T, crate::Error> {
    let format = format.as_ref();
    let value = match format {
        "bincode" => {
            let formatter = bincode::Bincode;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        "json" => {
            let formatter = json::Json;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        "toml" => {
            let formatter = toml::Toml;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        "toon" => {
            let formatter = toon::Toon;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        "plaintext" => {
            let formatter = plaintext::Plaintext;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        _ => {
            return Err(crate::Error::UnknownFormat {
                format: format.to_string(),
            });
        }
    };
    Ok(value)
}

pub fn encode_bytes<T: TransferObject>(
    format: impl AsRef<str>,
    value: &T,
) -> Result<Bytes, crate::Error> {
    let format = format.as_ref();
    let bytes = match format {
        "bincode" => {
            let formatter = bincode::Bincode;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        "json" => {
            let formatter = json::Json;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        "toml" => {
            let formatter = toml::Toml;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        "toon" => {
            let formatter = toon::Toon;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        "plaintext" => {
            let formatter = plaintext::Plaintext;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        _ => {
            return Err(crate::Error::UnknownFormat {
                format: format.to_string(),
            });
        }
    };
    Ok(bytes)
}
