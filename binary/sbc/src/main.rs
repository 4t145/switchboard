use std::path::PathBuf;

use clap::Parser;
use switchboard_controller::config::ControllerConfig;
use switchboard_model::custom_config::{FsLinkResolver, LinkOrValue, SerdeValue};

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
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let controller_config = retrieve_controller_config().await?;
    // fs load switchboard config
    tracing::debug!("Controller config: {:?}", controller_config);

    // let sb_config = {
    //     let path = &controller_config.resolve.fs.path;
    // };
    let context = switchboard_controller::ControllerContext::new(controller_config).await?;
    context.startup().await?;
    // let sb_config = context.resolve_config_from_fs().await?;
    // tracing::debug!("Resolved switchboard config: {:?}", sb_config);
    // context.update_config(sb_config).await?;
    tracing::info!("Controller started, press Ctrl+C to exit");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Controller shutting down");
    context.shutdown().await?;
    Ok(())
}
