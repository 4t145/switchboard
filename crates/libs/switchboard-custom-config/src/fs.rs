use std::ffi::OsStr;

use crate::{Link, link::LinkResolver};

pub fn detect_format_from_path(path: &std::path::Path) -> &[u8] {
    match path.extension().and_then(OsStr::to_str) {
        Some("bincode") => b"bincode",
        Some("json") => b"json",
        Some("toml") => b"toml",
        Some("toon") => b"toon",
        _ => {
            // decode as plaintext by default
            b"plaintext"
        }
    }
}

impl crate::CustomConfig {
    pub async fn read_from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        // read file type
        let format = detect_format_from_path(path);
        let data = tokio::fs::read(path).await?;
        Ok(crate::CustomConfig::new(format, data))
    }
    pub async fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let (format, parts) = self.clone().into_parts();
        let format_str = String::from_utf8(format).unwrap_or_default();
        let mut path = path.to_path_buf();
        path.set_extension(format_str);
        tokio::fs::write(path, parts).await
    }
}

pub struct FsLinkResolver;
impl LinkResolver for FsLinkResolver {
    type Error = std::io::Error;

    async fn fetch(&self, link: &Link) -> Result<crate::CustomConfig, Self::Error> {
        if link.0.starts_with("file://") {
            let path_str = &link.0[7..];
            let path = std::path::Path::new(path_str);
            crate::CustomConfig::read_from_file(path).await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported link scheme",
            ))
        }
    }

    async fn upload(&self, link: &Link, config: &crate::CustomConfig) -> Result<(), Self::Error> {
        if link.0.starts_with("file://") {
            let path_str = &link.0[7..];
            let path = std::path::Path::new(path_str);
            config.save_to_file(path).await
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported link scheme",
            ))
        }
    }
}
impl Link {}
