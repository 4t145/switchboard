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
            .name("mc-pf")
            .config("192.168.1.5:25565")
            .description("forward minecraft traffic")
            .provider("pf")
            .build(),
    );
    config.add_bind(
        Bind::builder()
            .addr("[::]:25565".parse()?)
            .description("mc")
            .service(ServiceDescriptor::named("mc-pf"))
            .build(),
    );
    config.add_bind(
        Bind::builder()
            .addr("[::]:10999".parse()?)
            .description("socks5 proxy")
            .service(AnonServiceDescriptor::builder().provider("socks5").build())
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
