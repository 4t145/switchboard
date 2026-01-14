use std::net::{IpAddr, Ipv6Addr};

use serde::{Deserialize, Serialize};
use switchboard_model::{FileStyleTlsResolver, Tls, kernel::HTTP_DEFAULT_PORT};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HttpListenerConfig {
    pub port: u16,
    pub host: IpAddr,
    pub tls: Option<Tls<FileStyleTlsResolver>>,
}

impl Default for HttpListenerConfig {
    fn default() -> Self {
        HttpListenerConfig {
            port: HTTP_DEFAULT_PORT,
            host: IpAddr::V6(Ipv6Addr::LOCALHOST),
            tls: None,
        }
    }
}
