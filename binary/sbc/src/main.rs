use std::path::PathBuf;

use clap::Parser;
use switchboard_controller::config::ControllerConfig;

#[derive(clap::Parser)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub command: CliSubCommand,
}

/// sbk serve -b [::]:8080 -s pf/[::]:9090
#[derive(clap::Subcommand)]
pub enum CliSubCommand {
    Start(CliSubCommandConfig),
}

#[derive(clap::Args)]
pub struct CliSubCommandConfig {
    config: PathBuf,
}

pub async fn retrieve_controller_config() -> Result<ControllerConfig, Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    match args.command {
        CliSubCommand::Start(cmd) => {
            let path = cmd.config;
            let config_str = tokio::fs::read_to_string(path).await?;
            let config: ControllerConfig = toml::from_str(&config_str)?;
            Ok(config)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    let config = retrieve_controller_config().await?;
    tracing::debug!("Controller config: {:?}", config);
    let context = switchboard_controller::ControllerContext::new(config);
    context.startup().await?;
    context.take_over_all_kernels().await?;
    tracing::info!("Controller started, press Ctrl+C to exit");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Controller shutting down");
    context.shutdown().await?;
    Ok(())
}
