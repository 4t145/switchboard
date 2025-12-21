use std::net::{Ipv6Addr, SocketAddr};

use tokio::net::TcpStream;
pub type TcpTranspose = super::FramedStreamTranspose<TcpStream>;

pub struct TcpTransposeConfig {
    pub addr: SocketAddr,
    pub max_frame_size: u32,
}

impl Default for TcpTransposeConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from((Ipv6Addr::LOCALHOST, 8056)),
            max_frame_size: 1 << 22,
        }
    }
}
impl TcpTranspose {
    pub async fn connect(config: TcpTransposeConfig) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(&config.addr).await?;
        Ok(TcpTranspose::new_with_max_frame_size(
            stream,
            config.max_frame_size,
        ))
    }
}
