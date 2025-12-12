use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use switchboard_model::TlsCertParams;

use crate::dir::app_dir;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct FileTlsResourceConfig {
    pub discovery_from_dirs: Vec<PathBuf>,
}

impl Default for FileTlsResourceConfig {
    fn default() -> Self {
        Self {
            discovery_from_dirs: vec![app_dir().join("tls")],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileTlsCertResourceError {
    #[error("IO error occurred")]
    IoError(#[from] std::io::Error),
    #[error("fail to parse PEM file")]
    CertParamsError(#[from] switchboard_model::TlsCertParamsError),
}

impl crate::ControllerContext {
    pub async fn discovery_tls_cert_from_file(
        &self,
    ) -> Result<BTreeMap<String, FileTlsCertResource>, FileTlsCertResourceError> {
        let mut results = BTreeMap::new();
        let Some(config) = &self.controller_config.resource_config.tls.file else {
            return Ok(results);
        };
        for dir in config.discovery_from_dirs.iter() {
            let entries = tokio::fs::read_dir(dir).await?;
            tokio::pin!(entries);
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file()
                    && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                {
                    if let Some(tls_name) = file_name.strip_suffix(".cert.pem") {
                        let key_path = path.with_file_name(format!("{}.key.pem", tls_name));
                        if key_path.exists() {
                            let resource = FileTlsCertResource {
                                cert_path: path.clone(),
                                key_path,
                            };
                            results.insert(tls_name.to_string(), resource);
                        }
                    }
                } else if path.is_dir() {
                    // check "cert.pem" and "key.pem" files inside the directory
                    let cert_path = path.join("cert.pem");
                    let key_path = path.join("key.pem");
                    if cert_path.exists()
                        && key_path.exists()
                        && let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                    {
                        let resource = FileTlsCertResource {
                            cert_path: cert_path.clone(),
                            key_path: key_path.clone(),
                        };
                        results.insert(dir_name.to_string(), resource);
                    }
                }
            }
        }
        Ok(results)
    }
}

impl FileTlsCertResource {
    pub async fn fetch(&self) -> Result<TlsCertParams, FileTlsCertResourceError> {
        let cert_bytes = tokio::fs::read(&self.cert_path).await?;
        let key_bytes = tokio::fs::read(&self.key_path).await?;
        let params = TlsCertParams::from_pem_file(&cert_bytes, &key_bytes)?;
        Ok(params)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct FileTlsCertResource {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}
