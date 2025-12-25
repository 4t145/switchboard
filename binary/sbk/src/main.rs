use std::path::PathBuf;
mod register;
use clap::Parser;
use switchboard_kernel::{KernelContext, config::KernelConfig};

#[derive(clap::Parser)]
pub struct CliArgs {
    config: PathBuf,
}

pub async fn retrieve_kernel_config() -> Result<KernelConfig, Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    let path = args.config;
    let config_str = tokio::fs::read_to_string(path).await?;
    let config: KernelConfig = toml::from_str(&config_str)?;
    Ok(config)
}
#[cfg(unix)]
pub async fn listen_reload_config_signal(
    context: KernelContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reload_signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;
    loop {
        reload_signal.recv().await;
        tracing::info!("SIGHUP received, reloading kernel config",);
        if let Some(config) = context.fetch_config_locally().await? {
            context.update_config(config).await?;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("debug,switchboard-http=trace")
        .init();
    let kernel_config = retrieve_kernel_config().await?;

    tracing::debug!("Starting kernel with config: {:?}", kernel_config);
    let context = KernelContext::new(kernel_config);
    #[cfg(unix)]
    {
        // listen for SIGHUP to reload config
        tokio::spawn({
            let context = context.clone();
            async move {
                if let Err(e) = listen_reload_config_signal(context).await {
                    tracing::error!("Error listening for reload config signal: {}", e);
                }
            }
        });
    }
    register::register_prelude(&context).await;
    tracing::info!("Kernel starting up...");
    context.startup().await?;
    tracing::info!("Kernel startup complete");
    tracing::info!("Kernel running, press Ctrl+C to exit");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C signal handler");
    tracing::info!("Ctrl+C signal received, shutting down...");
    context.shutdown().await;
    Ok(())
}
