use std::path::PathBuf;

use clap::Parser;
use switchboard_controller::{config::ControllerConfig, run::RunMode};

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
    #[clap(short, long)]
    config: PathBuf,
    /// disable the web root in config, would be useful for frontend dev
    #[clap(long, default_value_t = false)]
    no_web_root: bool,
    #[clap(long, default_value_t = false, alias = "kubernets")]
    k8s: bool,
}

pub async fn retrieve_controller_config(
    args: &CliArgs,
) -> Result<ControllerConfig, Box<dyn std::error::Error>> {
    match &args.command {
        CliSubCommand::Start(cmd) => {
            let path = &cmd.config;
            let config_str = tokio::fs::read_to_string(path).await?;
            let config: ControllerConfig = toml::from_str(&config_str)?;
            Ok(config)
        }
    }
}
pub fn is_in_k8s(args: &CliArgs) -> bool {
    match &args.command {
        CliSubCommand::Start(cmd) => cmd.k8s,
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let args = CliArgs::parse();
    let controller_config = retrieve_controller_config(&args).await?;
    // fs load switchboard config
    tracing::debug!("Controller config: {:?}", controller_config);
    // let sb_config = {
    //     let path = &controller_config.resolve.fs.path;
    // };
    let context = switchboard_controller::ControllerContext::new(controller_config).await?;
    let run_mode = if is_in_k8s(&args) {
        RunMode::K8s
    } else {
        RunMode::Standalone
    };
    context.startup(run_mode).await?;
    // let sb_config = context.resolve_config_from_fs().await?;
    // tracing::debug!("Resolved switchboard config: {:?}", sb_config);
    // context.update_config(sb_config).await?;
    tracing::info!("Controller started, press Ctrl+C to exit");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Controller shutting down");
    context.shutdown().await?;
    Ok(())
}
