use std::path::PathBuf;

use ::clap::Parser;

use crate::workspace::Workspace;

mod clap;
mod commands;
mod workspace;
mod error;

pub use error::Error;
pub use error::Result;

pub struct Context {
    sbk_path: PathBuf,
    workspace: Workspace,
}

#[tokio::main]
async fn main() {
    let args = clap::ClapArgs::parse();
    
    println!("{args:?}")
}
// just sbk
