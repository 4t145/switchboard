use std::net::SocketAddr;

use switchboard_model::Tls;
use switchboard_payload::BytesPayload;
use switchboard_service::tcp::RunningTcpService;

use super::SupervisorError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TcpServiceInfo {
    // id is the unique identifier of the TCP service
    pub id: String,
    // if bind and tls config changed, we need to rebind the service
    pub bind: SocketAddr,
    pub tls_config: Option<Tls>,
    // if provider or config changed, we can update inner service
    pub provider: String,
    pub config: Option<BytesPayload>,
    // if name or description changed, we can just update metadata
    pub name: Option<String>,
    pub bind_description: Option<String>,
    pub service_description: Option<String>,
}

#[derive(Debug)]
pub struct TcpServiceHandle {
    pub service: Result<RunningTcpService, SupervisorError>,
    pub info: TcpServiceInfo,
}
