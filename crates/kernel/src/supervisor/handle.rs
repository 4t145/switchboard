use std::{net::SocketAddr, sync::Arc};

use switchboard_model::Tls;
use switchboard_service::tcp::RunningTcpService;

use super::SupervisorError;

#[derive(Debug)]
pub struct TcpServiceInfo {
    pub id: String,
    pub name: Option<String>,
    pub bind: SocketAddr,
    pub provider: String,
    pub config: Option<String>,
    pub bind_description: Option<String>,
    pub service_description: Option<String>,
    pub tls_config: Option<Tls>,
}

#[derive(Debug)]
pub struct TcpServiceHandle {
    pub service: Result<RunningTcpService, SupervisorError>,
    pub info: TcpServiceInfo,
}
