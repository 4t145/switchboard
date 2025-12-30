use std::path::PathBuf;
const APP_NAME: &str = "switchboard";
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/var/lib"))
        .join(APP_NAME)
}

pub fn config_local_db_path() -> PathBuf {
    data_dir().join("config.db")
}
