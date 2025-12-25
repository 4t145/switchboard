use bytes::Bytes;
pub const BINCODE: &str = "bincode";
pub const JSON: &str = "json";
pub const TOML: &str = "toml";
pub const TOON: &str = "toon";
pub const PLAINTEXT: &str = "plaintext";
// pub const BINCODE_FORMAT: bincode::Bincode = bincode::Bincode;
// pub const JSON_FORMAT: json::Json = json::Json;
// pub const TOML_FORMAT: toml::Toml = toml::Toml;
// pub const TOON_FORMAT: toon::Toon = toon::Toon;
// pub const PLAINTEXT_FORMAT: plaintext::Plaintext = plaintext::Plaintext;
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
        BINCODE => {
            let formatter = bincode::Bincode;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        JSON => {
            let formatter = json::Json;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        TOML => {
            let formatter = toml::Toml;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        TOON => {
            let formatter = toon::Toon;
            formatter
                .decode_bytes::<T>(bytes)
                .map_err(|e| DecodeError::new(e, formatter.format_name()))?
        }
        PLAINTEXT => {
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
        BINCODE => {
            let formatter = bincode::Bincode;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        JSON => {
            let formatter = json::Json;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        TOML => {
            let formatter = toml::Toml;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        TOON => {
            let formatter = toon::Toon;
            formatter
                .encode_bytes::<T>(value)
                .map_err(|e| EncodeError::new(e, formatter.format_name()))?
        }
        PLAINTEXT => {
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
