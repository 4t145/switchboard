use supervisor::{Supervisor, TcpServiceInfo};
use switchboard_http::HttpProvider;
use switchboard_model::{ConfigService, NamedService, ServiceDescriptor};
use switchboard_pf::PortForwardProvider;
use switchboard_service::registry::ServiceProviderRegistry;
use switchboard_socks5::Socks5Provider;
use switchboard_uds::UdsProvider;

pub mod clap;
pub mod config;
pub mod supervisor;
pub mod tls;
pub mod script_engine;
pub use switchboard_model as model;

pub fn register_prelude(registry: &mut ServiceProviderRegistry) {
    // Register the prelude services
    registry.register_tcp_provider(Socks5Provider);
    registry.register_tcp_provider(PortForwardProvider);
    registry.register_tcp_provider(HttpProvider);
    registry.register_tcp_provider(UdsProvider);
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
    let sb_config = config_service
        .fetch_config()
        .await
        .map_err(Error::ConfigError)?;

    {
        let _registry = registry.read().await;
        for (id, bind) in sb_config.get_enabled() {
            tracing::info!(%id, %bind, "Adding bind to supervisor");
            let sd = &bind.service;
            let service_info = match sd {
                ServiceDescriptor::Anon(anon_service_descriptor) => {
                    let mut tls_config = None;
                    if let Some(tls_name) = &anon_service_descriptor.tls {
                        tls_config = sb_config.get_tls(tls_name);
                    }

                    TcpServiceInfo {
                        id: id.to_owned(),
                        bind: bind.addr,
                        bind_description: bind.description.clone(),
                        config: anon_service_descriptor.config.clone(),
                        provider: anon_service_descriptor.provider.clone(),
                        name: None,
                        service_description: None,
                        tls_config: tls_config.cloned(),
                    }
                }
                ServiceDescriptor::Named(name) => {
                    let Some(NamedService {
                        provider,
                        name,
                        config,
                        description,
                        tls,
                    }) = sb_config.get_named_service(name)
                    else {
                        tracing::error!(%id, %name, "Failed to get named service");
                        continue;
                    };
                    let mut tls_config = None;
                    if let Some(tls_name) = tls {
                        tls_config = sb_config.get_tls(tls_name);
                    }
                    TcpServiceInfo {
                        id: id.to_owned(),
                        bind: bind.addr,
                        bind_description: bind.description.clone(),
                        config: config.clone(),
                        provider: provider.clone(),
                        name: Some(name.clone()),
                        service_description: description.clone(),
                        tls_config: tls_config.cloned(),
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
