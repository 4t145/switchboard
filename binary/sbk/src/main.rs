use std::path::PathBuf;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("debug,switchboard-http=trace")
        .init();
    let config = retrieve_kernel_config().await?;
    tracing::debug!("Starting kernel with config: {:?}", config);
    let context = KernelContext::new(config);
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
