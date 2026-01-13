use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};
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
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct FileTcpServiceConfig<L> {
    pub provider: String,
    pub name: String,
    pub config: Option<LinkOrValue<L, SerdeValue>>,
    pub description: Option<String>,
    pub binds: Vec<FileBind>,
}

#[derive(Clone, Debug, Serialize, bincode::Encode, bincode::Decode)]
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

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(default)]
pub struct FileStyleConfig<L> {
    pub tcp_services: Vec<FileTcpServiceConfig<L>>,
    pub tls: Vec<FileStyleTls<UnresolvedFileStyleTlsResolver<L>>>,
}

impl<T> Default for FileStyleConfig<T> {
    fn default() -> Self {
        Self {
            tcp_services: Vec::new(),
            tls: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct FileStyleTls<Tls = FileStyleTlsResolver> {
    pub name: String,
    #[serde(flatten)]
    pub resolver: Tls,
    #[serde(default)]
    pub options: Option<TlsOptions>,
}

use switchboard_custom_config::SerdeValue;

#[derive(Debug, thiserror::Error)]
pub enum ResolveConfigFileError {
    #[error("Read file error")]
    Resolve {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },
    #[error("Value deserialize error")]
    ValueDeserialize(#[from] switchboard_custom_config::SerdeValueError),
    #[error("config error")]
    ConfigError(#[from] switchboard_custom_config::Error),
    #[error("Resolve tls cert error")]
    ResolveTlsCert {
        #[from]
        source: crate::tls::TlsResolveError,
    },
}

impl ResolveConfigFileError {
    pub const fn when_resolve<E>(context: &'static str) -> impl FnOnce(E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        move |source| Self::Resolve {
            source: Box::new(source),
            context: context.to_string(),
        }
    }
}

impl<L> FileStyleConfig<L> {
    pub async fn resolve_into_standard<R>(
        self,
        resolver: &R,
    ) -> Result<crate::ServiceConfig, ResolveConfigFileError>
    where
        L: Send + Sync + Clone + DeserializeOwned + 'static,
        R: Resolver<L, SerdeValue> + Resolver<L, PemFile> + Resolver<L, PemsFile>,
    {
        let config = self;
        let mut resolved_tcp_services = std::collections::BTreeMap::new();
        let mut tcp_listeners = std::collections::BTreeMap::new();
        let mut tcp_routes = BTreeMap::new();
        let mut task_set =
            tokio::task::JoinSet::<Result<(String, Tls<TlsResolver>), TlsResolveError>>::new();

        for file_tls in config.tls.into_iter() {
            let name = file_tls.name;
            let options = file_tls.options;
            let tls_resolver = file_tls.resolver;
            let resolver = resolver.clone();
            task_set.spawn(async move {
                let resolved_tls_resolver = tls_resolver.resolve_to_standard(&resolver).await?;
                Ok((
                    name.clone(),
                    Tls {
                        resolver: resolved_tls_resolver,
                        options: options,
                    },
                ))
            });
        }
        let tls = task_set
            .join_all()
            .await
            .into_iter()
            .collect::<Result<_, _>>()?;
        for service_config in config.tcp_services.into_iter() {
            let service_name = service_config.name.clone();
            let resolved_config = if let Some(link) = &service_config.config {
                let resolved: SerdeValue = link.clone().resolve_with(resolver).await.map_err(
                    ResolveConfigFileError::when_resolve("resolve service config"),
                )?;
                let resolved =
                    fs_preprocess_service_config(&service_config.provider, resolver, resolved)
                        .await?;
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
        let config = ServiceConfig {
            tcp_services: resolved_tcp_services,
            tcp_listeners,
            tcp_routes,
            tls,
        };
        Ok(config)
    }
}

pub async fn fetch_config<L, R>(
    service_config: LinkOrValue<L, SerdeValue>,
    resolver: &R,
) -> Result<crate::ServiceConfig, ResolveConfigFileError>
where
    L: Send + Sync + Clone + DeserializeOwned + 'static,
    R: Resolver<L, SerdeValue> + Resolver<L, PemFile> + Resolver<L, PemsFile>,
{
    let config: SerdeValue = service_config.resolve_with(resolver).await.map_err(
        ResolveConfigFileError::when_resolve("resolve service config"),
    )?;
    let config: FileStyleConfig<L> = config.deserialize_into()?;
    let resolved_config = config.resolve_into_standard(resolver).await?;
    Ok(resolved_config)
}

pub async fn fs_preprocess_service_config<L, R: Resolver<L, SerdeValue>>(
    provider: &str,
    resolver: &R,
    resolved_config: SerdeValue,
) -> Result<SerdeValue, ResolveConfigFileError>
where
    L: Send + Sync + 'static,
    crate::services::http::Config<LinkOrValue<L, SerdeValue>>: DeserializeOwned,
{
    match provider {
        "http" => {
            let http_config = resolved_config
                .deserialize_into::<crate::services::http::Config<LinkOrValue<L, SerdeValue>>>()?;
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
                let actual_config = instance_data.config.resolve_with(resolver).await.map_err(
                    ResolveConfigFileError::when_resolve("resolve http instance config"),
                )?;
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
            let encoded_config = SerdeValue::serialize_from(&resolved_config)?;
            Ok(encoded_config)
        }
        _ => Ok(resolved_config.clone()),
    }
}
