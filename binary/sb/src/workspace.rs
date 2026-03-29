use std::path::PathBuf;

use tokio::fs::ReadDir;
// sb workspace organization
// 
// 
// 
pub struct Workspace {
    dir: PathBuf,
}


impl Workspace {
    pub fn kernel_config_path(&self) -> PathBuf {
        self.dir.join("kernel.toml")
    }
    
    pub fn service_config_path(&self) -> PathBuf {
        self.dir.join("config.toml")
    }
    
    pub fn run_file(&self) -> PathBuf {
        self.dir.join(".run")
    }
    
    pub fn pid_file(&self) -> PathBuf {
        self.dir.join(".pid")
    }
}
