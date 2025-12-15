use std::ffi::OsStr;

impl crate::BytesPayload {
    pub async fn read_from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        // read file type
        let format = match path.extension().and_then(OsStr::to_str) {
            Some("bincode") => "bincode",
            Some("json") => "json",
            Some("toml") => "toml",
            Some("toon") => "toon",
            _ => {
                // decode as plaintext by default
                "plaintext"
            }
        };

        let data = tokio::fs::read(path).await?;
        Ok(crate::BytesPayload::new(format, data))
    }
}
