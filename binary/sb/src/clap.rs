use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct ClapArgs {
    #[arg(short, long, env("SB_WORKSPACE"))]
    pub workspace: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Config { config: PathBuf },
    Start,
    Reload { config: Option<PathBuf> },
    Shutdown,
    Status,
}
