use std::path::PathBuf;

pub fn app_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/var/lib"))
        .join("switchboard")
}
