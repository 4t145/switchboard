use std::ffi::OsStr;

use crate::{Link, link::LinkResolver};

pub fn detect_format_from_path(path: &std::path::Path) -> &str {
    match path.extension().and_then(OsStr::to_str) {
        Some("bincode") => "bincode",
        Some("json") => "json",
        Some("toml") => "toml",
        Some("toon") => "toon",
        _ => {
            // decode as plaintext by default
            "plaintext"
        }
    }
}

impl<V> crate::ConfigWithFormat<V>
where
    V: crate::formats::TransferObject,
{
    pub async fn read_from_file(path: &std::path::Path) -> Result<Self, crate::Error> {
        // read file type
        let format = detect_format_from_path(path);
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| crate::Error::resolve_error(e, Link::from(path.to_path_buf())))?;
        crate::ConfigWithFormat::decode(format, data.into())
    }
    pub async fn save_to_file(&self, path: &std::path::Path) -> Result<(), crate::Error> {
        let mut path = path.to_path_buf();
        path.set_extension(&self.format);
        let bytes = self.encode()?;
        tokio::fs::write(&path, bytes)
            .await
            .map_err(|e| crate::Error::resolve_error(e, Link::from(path.to_path_buf())))?;
        Ok(())
    }
}
#[derive(Clone, Debug, Default)]
pub struct FsLinkResolver;
impl LinkResolver for FsLinkResolver {
    async fn fetch<V: crate::formats::TransferObject>(
        &self,
        link: &Link,
    ) -> Result<crate::ConfigWithFormat<V>, crate::Error> {
        if let Some(path) = link.as_file_path() {
            crate::ConfigWithFormat::read_from_file(path).await
        } else {
            Err(crate::Error::resolve_error("Not a file link", link.clone()))
        }
    }

    // async fn upload<V: crate::formats::TransferObject >(&self, link: &Link, config: &crate::ConfigWithFormat<V>) -> Result<(), crate::Error> {
    //     if let Some(path) = link.as_file_path() {
    //         config.save_to_file(path).await
    //     } else {
    //         Err(crate::Error::resolve_error("Not a file link", link.clone()))
    //     }
    // }
}
impl Link {}
