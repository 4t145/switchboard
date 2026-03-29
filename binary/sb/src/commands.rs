use std::{path::PathBuf, process::ExitStatus};

use tokio::fs;

use crate::{Context, clap::Commands};

impl Context {
    pub async fn exec(&self, commands: Commands) -> crate::Result<ExitStatus> {
        match commands {
            Commands::Config { config } => self.config(Some(config)).await,
            Commands::Reload { config } => self.reload(config).await,
            Commands::Shutdown => self.kill().await,
            Commands::Start => self.start().await,
            _ => return Err(crate::Error::Unimplemented),
        }
    }
    async fn start(&self) -> crate::Result<ExitStatus> {
        let task = tokio::process::Command::new(&self.sbk_path)
            .kill_on_drop(false)
            .spawn()?;
        if let Some(pid) = task.id() {
            println!("{pid}")
        }
        Ok(ExitStatus::default())
    }
    async fn config(&self, config_path: Option<PathBuf>) -> crate::Result<ExitStatus> {
        self.reset_service_config(config_path).await?;
        Ok(ExitStatus::default())
    }
    async fn kill(&self) -> crate::Result<ExitStatus> {
        let pid = self.read_pid().await?;
        let mut kill_task = tokio::process::Command::new("kill")
            .arg("-9")
            .arg(pid)
            .spawn()?;
        let status = kill_task.wait().await?;
        Ok(status)
    }
    async fn reload(&self, config_path: Option<PathBuf>) -> crate::Result<ExitStatus> {
        if config_path.is_some() {
            self.reset_service_config(config_path).await?;
        }
        let pid = self.read_pid().await?;
        let mut kill_task = tokio::process::Command::new("kill")
            .arg("-1")
            .arg(pid)
            .spawn()?;
        let status = kill_task.wait().await?;
        Ok(status)
    }
}

impl Context {
    async fn reset_service_config(&self, new_path: Option<PathBuf>) -> crate::Result<()> {
        let kernel_config_path = self.workspace.kernel_config_path();
        let file = fs::read_to_string(&kernel_config_path).await?;
        let mut toml_ast: toml_edit::DocumentMut = toml_edit::Document::parse(file)?.into_mut();
        let Some(item) = toml_ast.get_mut("config") else {
            return Ok(());
        };
        if let Some(new_path) = new_path {
            let new_path_str = new_path
                .to_str()
                .expect(&format!("invalid path {new_path:?}"));
            *item = toml_edit::Item::Value(toml_edit::Value::String(toml_edit::Formatted::new(
                format!("file://{new_path_str}"),
            )));
        }
        Ok(())
    }
    async fn read_pid(&self) -> crate::Result<String> {
        let pid = fs::read_to_string(self.workspace.pid_file()).await?;
        Ok(pid)
    }
}
