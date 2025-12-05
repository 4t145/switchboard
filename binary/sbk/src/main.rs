use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use switchboard_kernel::{
    KernelContext,
    config::{KernelConfig, mem::MemConfig},
    controller::ControllerConfig,
    model::*,
};
#[derive(clap::Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub command: CliSubCommand,
}

/// sbk serve -b [::]:8080 -s pf/[::]:9090
#[derive(clap::Subcommand)]
pub enum CliSubCommand {
    Serve(CliSubCommandServe),
    Config(CliSubCommandConfig),
}

#[derive(clap::Args)]
pub struct CliSubCommandServe {
    #[clap(long, short, default_value = "[::]:8080")]
    pub bind: String,
    #[clap(long, short, default_value = "pf/[::]:9090")]
    pub service: String,
}

#[derive(clap::Args)]
pub struct CliSubCommandConfig {
    path: PathBuf,
}

pub async fn retrieve_kernel_config() -> Result<KernelConfig, Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    match args.command {
        CliSubCommand::Config(cmd) => {
            let path = cmd.path;
            let config_str = tokio::fs::read_to_string(path).await?;
            let config: KernelConfig = toml::from_str(&config_str)?;
            Ok(config)
        }
        CliSubCommand::Serve(cmd) => {
            let mut config = MemConfig::new();
            let service = ServiceDescriptor::from_str(&cmd.service)?;
            config.add_bind(
                Bind::builder()
                    .addr(cmd.bind.parse()?)
                    .description("cli bind")
                    .service(service)
                    .build(),
            );

            Ok(KernelConfig {
                info: kernel::KernelInfo::default(),
                controller: ControllerConfig::default(),
                startup: config.into_inner(),
            })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
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
