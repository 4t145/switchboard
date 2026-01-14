use std::net::SocketAddr;

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    bincode::Encode,
    bincode::Decode,
    PartialEq,
    Eq,
)]

pub struct TcpRoute {
    pub bind: SocketAddr,
    pub service: String,
    pub tls: Option<String>,
}
