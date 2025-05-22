use switchboard_kernel::{KernelContext, config::mem::Mem};
use switchboard_model::{AnonServiceDescriptor, Bind, NamedService, ServiceDescriptor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut config = Mem::new();
    config.add_named_service(
        NamedService::builder()
            .name("ssh-pf")
            .config("[::]:10022")
            .description("forward ssh from 10022")
            .provider("pf")
            .build(),
    );
    config.add_bind(
        Bind::builder()
            .addr("[::]:20222".parse()?)
            .description("ssh")
            .service(ServiceDescriptor::Named("ssh-pf".to_string()))
            .build(),
    );
    config.add_bind(
        Bind::builder()
            .addr("[::]:10999".parse()?)
            .description("socks5 proxy")
            .service(ServiceDescriptor::Anon(
                AnonServiceDescriptor::builder()
                    .provider("socks5")
                    .build(),
            ))
            .build(),
    );
    let mut context = KernelContext::startup(config).await?;
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C signal handler");
    tracing::info!("Ctrl+C signal received, shutting down...");
    context.supervisor.shutdown().await;
    Ok(())
}
