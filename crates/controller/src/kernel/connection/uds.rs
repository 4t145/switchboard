use std::path::PathBuf;

use switchboard_model::kernel::UDS_DEFAULT_PATH;
use tokio::net::UnixStream;
pub type UdsTranspose = super::FramedStreamTranspose<UnixStream>;

pub struct UdsTransposeConfig {
    pub path: PathBuf,
    pub max_frame_size: u32,
}

impl Default for UdsTransposeConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from(UDS_DEFAULT_PATH),
            max_frame_size: 1 << 22,
        }
    }
}
impl UdsTranspose {
    pub async fn connect(config: UdsTransposeConfig) -> Result<Self, std::io::Error> {
        let stream = UnixStream::connect(&config.path).await?;
        Ok(UdsTranspose::new_with_max_frame_size(
            stream,
            config.max_frame_size,
        ))
    }
}
