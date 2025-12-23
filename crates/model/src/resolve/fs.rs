use std::{
    collections::BTreeMap,
    net::SocketAddr,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct FsResolveConfig {
    #[serde(default = "default_switchboard_config_path")]
    pub path: PathBuf,
}

impl Default for FsResolveConfig {
    fn default() -> Self {
        Self {
            path: default_switchboard_config_path(),
        }
    }
}

pub fn default_switchboard_config_path() -> PathBuf {
    if cfg!(target_os = "linux") {
        PathBuf::from("~/.switchboard/config.toml")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(r"%USERPROFILE%\.switchboard\config.toml")
    } else if cfg!(target_os = "macos") {
        PathBuf::from("~/Library/Application Support/switchboard/config.toml")
    } else {
        PathBuf::from("./config.toml")
    }
}

use crate::{services::http::InstanceType, *};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
pub struct FileTcpServiceConfig {
    pub provider: String,
    pub name: String,
    pub config: Option<Link>,
    pub description: Option<String>,
    pub binds: Vec<FileBind>,
}

#[derive(Clone, Debug, Serialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq)]
pub struct FileBind {
    pub bind: SocketAddr,
    pub tls: Option<String>,
    pub description: Option<String>,
}

impl<'de> Deserialize<'de> for FileBind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct FileBindStruct {
            pub bind: SocketAddr,
            pub tls: Option<String>,
        }
        pub struct FileBindVisitor;
        impl<'de> serde::de::Visitor<'de> for FileBindVisitor {
            type Value = FileBind;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or a struct representing a FileBind")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                FileBind::from_expr(v).map_err(serde::de::Error::custom)
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let fb_struct =
                    FileBindStruct::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(FileBind {
                    bind: fb_struct.bind,
                    tls: fb_struct.tls,
                    description: None,
                })
            }
        }
        deserializer.deserialize_any(FileBindVisitor)
    }
}

impl FileBind {
    pub fn from_expr(expr: &str) -> Result<Self, std::net::AddrParseError> {
        if let Some((addr_str, tls_str)) = expr.split_once(",") {
            let addr: SocketAddr = addr_str.trim().parse()?;
            let tls_str = tls_str.trim();
            let tls_str = tls_str.strip_prefix("tls=").unwrap_or(tls_str);
            let tls = Some(tls_str.trim().to_string());
            Ok(FileBind {
                bind: addr,
                tls,
                description: None,
            })
        } else {
            let addr: SocketAddr = expr.trim().parse()?;
            Ok(FileBind {
                bind: addr,
                tls: None,
                description: None,
            })
        }
    }
}

#[derive(
    Clone, Debug, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
pub struct FileConfig {
    #[serde(default)]
    pub tcp_services: Vec<FileTcpServiceConfig>,
    #[serde(default)]
    pub tls: Vec<FileTls>,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, Hash, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]
pub struct FileTls {
    pub name: String,
    #[serde(flatten)]
    pub resolver: TlsResolverInFile,
    #[serde(default)]
    pub options: Option<TlsOptions>,
}

use switchboard_custom_config::{CustomConfig, Link, LinkResolver};

#[derive(Debug, thiserror::Error)]
pub enum ResolveConfigFileError {
    #[error("Read file error")]
    ReadFile {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("Decode config error")]
    DecodeConfig(#[from] switchboard_custom_config::Error),
    #[error("Link resolution error")]
    LinkResolution {
        #[source]
        source: std::io::Error,
        link: String,
    },
    #[error("Resolve tls cert error")]
    ResolveTlsCert {
        #[source]
        source: TlsResolverLoadError,
        name: String,
    },
}

pub async fn fetch_config(
    root_config_path: &Path,
) -> Result<crate::Config, ResolveConfigFileError> {
    let bytes = tokio::fs::read(root_config_path).await.map_err(|source| {
        ResolveConfigFileError::ReadFile {
            source,
            path: root_config_path.to_path_buf(),
        }
    })?;
    let format = switchboard_custom_config::fs::detect_format_from_path(root_config_path);
    let config: FileConfig =
        switchboard_custom_config::formats::decode_bytes(format, bytes.into())?;
    let fs_link_resolver = switchboard_custom_config::fs::FsLinkResolver;
    let mut resolved_tcp_services = std::collections::BTreeMap::new();
    let mut tcp_listeners = std::collections::BTreeMap::new();
    let mut tcp_routes = BTreeMap::new();
    let mut tls = BTreeMap::new();
    for file_tls in config.tls.into_iter() {
        let name = file_tls.name;
        let options = file_tls.options;
        let tls_resolver = file_tls.resolver;
        tls.insert(
            name,
            Tls {
                resolver: tls_resolver,
                options: options.unwrap_or_default(),
            },
        );
    }
    for service_config in config.tcp_services.into_iter() {
        let service_name = service_config.name.clone();
        let resolved_config = if let Some(link) = &service_config.config {
            let resolved = fs_link_resolver.fetch(link).await.map_err(|source| {
                ResolveConfigFileError::LinkResolution {
                    source,
                    link: link.0.clone(),
                }
            })?;
            // specially handle http service config
            tracing::debug!(%service_name, %service_config.provider, "trying to preprocess service config");
            let resolved = fs_preprocess_service_config(&service_config.provider, resolved).await?;
            tracing::debug!(%service_name, %service_config.provider, "finished preprocessing service config");
            Some(resolved)
        } else {
            None
        };
        let resolved_service = crate::TcpServiceConfig {
            provider: service_config.provider,
            name: service_config.name,
            config: resolved_config,
            description: service_config.description,
        };
        for bind in service_config.binds {
            tcp_listeners.insert(
                bind.bind,
                crate::Listener {
                    bind: bind.bind,
                    description: bind.description.clone(),
                },
            );
            tcp_routes.insert(
                bind.bind,
                crate::tcp_route::TcpRoute {
                    bind: bind.bind,
                    service: service_name.clone(),
                    tls: bind.tls,
                },
            );
        }
        resolved_tcp_services.insert(service_name, resolved_service);
    }
    let config = Config {
        tcp_services: resolved_tcp_services,
        tcp_listeners,
        tcp_routes,
        tls,
    };
    let config = config.resolve_tls_with_skip().await;
    Ok(config)
}

pub async fn fs_preprocess_service_config(
    provider: &str,
    resolved_config: CustomConfig,
) -> Result<CustomConfig, ResolveConfigFileError> {
    match provider {
        "http" => {
            let http_config = resolved_config.decode::<crate::services::http::Config<Link>>()?;
            let mut new_instances = BTreeMap::new();
            for (instance_id, instance_data) in http_config
                .flow
                .instances
                .into_iter()
                .chain(
                    http_config
                        .flow
                        .filters
                        .into_iter()
                        .map(|(id, instance)| (id, instance.with_type(InstanceType::Filter))),
                )
                .chain(
                    http_config
                        .flow
                        .nodes
                        .into_iter()
                        .map(|(id, instance)| (id, instance.with_type(InstanceType::Node))),
                )
            {
                let resolver = switchboard_custom_config::fs::FsLinkResolver;
                let actual_config =
                    resolver
                        .fetch(&instance_data.config)
                        .await
                        .map_err(|source| ResolveConfigFileError::LinkResolution {
                            source,
                            link: instance_data.config.0.clone(),
                        })?;
                let resolved_instance_data = crate::services::http::InstanceData {
                    config: actual_config,
                    name: instance_data.name,
                    class: instance_data.class,
                    r#type: instance_data.r#type,
                };
                new_instances.insert(instance_id, resolved_instance_data);
            }
            let resolved_config = crate::services::http::Config {
                flow: crate::services::http::FlowConfig {
                    instances: new_instances,
                    entrypoint: http_config.flow.entrypoint,
                    nodes: BTreeMap::new(),
                    filters: BTreeMap::new(),
                    options: http_config.flow.options,
                },
                server: http_config.server,
            };
            let encoded_config = CustomConfig::encode("bincode", &resolved_config)?;
            return Ok(encoded_config);
        }
        _ => Ok(resolved_config.clone()),
    }
}
