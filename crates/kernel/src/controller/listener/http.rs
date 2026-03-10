use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use switchboard_model::{Tls, UnresolvedFileStyleTlsResolver, kernel::HTTP_DEFAULT_PORT};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HttpListenerConfig {
    pub port: u16,
    #[serde(alias = "addr")]
    pub host: IpAddr,
    pub endpoint: Option<String>,
    pub tls: Option<Tls<UnresolvedFileStyleTlsResolver<PathBuf>>>,
}

impl HttpListenerConfig {
    /// Get the endpoint to listen on. If endpoint is not set, construct it from host and port.
    pub fn get_endpoint(&self) -> String {
        if let Some(endpoint) = &self.endpoint {
            endpoint.clone()
        } else {
            let socket_addr = SocketAddr::new(self.host, self.port);
            format!("grpc://{socket_addr}")
        }
    }
}
impl Default for HttpListenerConfig {
    fn default() -> Self {
        let host: IpAddr = IpAddr::V6(Ipv6Addr::LOCALHOST);
        let socket_addr = SocketAddr::new(host, HTTP_DEFAULT_PORT);
        HttpListenerConfig {
            port: HTTP_DEFAULT_PORT,
            endpoint: Some(format!("grpc://{}", socket_addr)),
            host,
            tls: None,
        }
    }
}
