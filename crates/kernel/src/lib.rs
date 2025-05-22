use supervisor::{Supervisor, TcpServiceInfo};
use switchboard_model::{ConfigService, CursorQuery, Indexed, NamedService, ServiceDescriptor};
use switchboard_pf::PortForwardProvider;
use switchboard_service::registry::ServiceProviderRegistry;
use switchboard_socks5::Socks5Provider;

pub mod clap;
pub mod config;
pub mod supervisor;
pub mod tls;

pub fn register_prelude(registry: &mut ServiceProviderRegistry) {
    // Register the prelude services

    registry.register_tcp_provider(Socks5Provider);
    registry.register_tcp_provider(PortForwardProvider);
}
#[derive(Debug, thiserror::Error)]
pub enum Error<C: ConfigService> {
    #[error("Config service error: {0}")]
    ConfigError(C::Error),
}

pub async fn startup<C>(config_service: C) -> Result<KernelContext<C>, Error<C>>
where
    C: ConfigService,
{
    tracing::info!(
        "Starting up kernel with config: {config_type_name}",
        config_type_name = std::any::type_name::<C>()
    );
    let mut supervisor = Supervisor::new();
    let registry = ServiceProviderRegistry::global();
    // Register the prelude services
    {
        let mut guard = supervisor.registry.write().await;
        register_prelude(&mut guard);
    }
    let mut query = CursorQuery::first_page(64);
    let mut enabled_binds = Vec::new();
    loop {
        let enabled = config_service
            .get_enabled(query.clone())
            .await
            .map_err(Error::ConfigError)?;
        enabled_binds.extend(enabled.items);
        let Some(next_query) = query.next_page(enabled.next_cursor) else {
            break;
        };
        query = next_query;
    }
    {
        let _registry = registry.read().await;
        for Indexed { id, data: bind } in enabled_binds {
            tracing::info!(%id, %bind, "Adding bind to supervisor");
            let sd = bind.service;
            let service_info = match sd {
                ServiceDescriptor::Anon(anon_service_descriptor) => {
                    let mut tls_config = None;
                    if let Some(tls_name) = anon_service_descriptor.tls {
                        tls_config = config_service
                            .get_tls(tls_name)
                            .await
                            .map_err(Error::ConfigError)?;
                    }

                    TcpServiceInfo {
                        id,
                        bind: bind.addr,
                        bind_description: bind.description,
                        config: anon_service_descriptor.config,
                        provider: anon_service_descriptor.provider,
                        name: None,
                        service_description: None,
                        tls_config,
                    }
                }
                ServiceDescriptor::Named(name) => {
                    let Some(NamedService {
                        provider,
                        name,
                        config,
                        description,
                        tls,
                    }) = config_service
                        .get_named_service(name.clone())
                        .await
                        .map_err(Error::ConfigError)?
                    else {
                        tracing::error!(%id, %name, "Failed to get named service");
                        continue;
                    };
                    let mut tls_config = None;
                    if let Some(tls_name) = tls {
                        tls_config = config_service
                            .get_tls(tls_name)
                            .await
                            .map_err(Error::ConfigError)?;
                    }
                    TcpServiceInfo {
                        id,
                        bind: bind.addr,
                        bind_description: bind.description,
                        config,
                        provider,
                        name: Some(name),
                        service_description: description,
                        tls_config,
                    }
                }
            };
            supervisor.add_tcp_service(service_info).await;
        }
    }
    tracing::info!("Kernel startup complete");
    Ok(KernelContext {
        config: config_service,
        supervisor,
    })
}

pub struct KernelContext<C> {
    pub config: C,
    pub supervisor: Supervisor,
}

impl<C: ConfigService> KernelContext<C> {
    pub async fn startup(config: C) -> Result<Self, Error<C>> {
        startup(config).await
    }
}
