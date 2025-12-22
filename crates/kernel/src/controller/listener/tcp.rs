use std::net::{IpAddr, Ipv6Addr};

use serde::{Deserialize, Serialize};
use switchboard_model::control::{ControllerMessage, KernelMessage};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TcpListenerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: IpAddr,
    #[serde(default = "default_max_frame_size")]
    pub max_frame_size: u32,
}

const fn default_port() -> u16 {
    8056
}

const fn default_host() -> IpAddr {
    IpAddr::V6(Ipv6Addr::LOCALHOST)
}

const fn default_max_frame_size() -> u32 {
    1 << 22
}