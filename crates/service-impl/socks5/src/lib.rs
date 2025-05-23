use std::{
    convert::Infallible,
    net::{Ipv4Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
};

use switchboard_service::{
    TcpServiceProvider,
    tcp::{AsyncStream, TcpService},
};
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const CMD_CONNECT: u8 = 0x01;
const CMD_BIND: u8 = 0x02;
const CMD_UDP_ASSOCIATE: u8 = 0x03;
const ADDR_TYPE_IPV4: u8 = 0x01;
const ADDR_TYPE_IPV6: u8 = 0x04;
const ADDR_TYPE_DOMAIN: u8 = 0x03;

const REP_SUCCESS: u8 = 0x00;
const REP_GENERAL_SOCKS_SERVER_FAILURE: u8 = 0x01;
const REP_CONNECTION_NOT_ALLOWED: u8 = 0x02;
const REP_NETWORK_UNREACHABLE: u8 = 0x03;
const REP_HOST_UNREACHABLE: u8 = 0x04;
const REP_CONNECTION_REFUSED: u8 = 0x05;
const REP_TTL_EXPIRED: u8 = 0x06;
const REP_COMMAND_NOT_SUPPORTED: u8 = 0x07;
const REP_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;

const RSV: u8 = 0x00;
pub enum Socks5Request {
    Connect(Socks5Addr),
    Bind(Socks5Addr),
    UdpAssociate(Socks5Addr),
}

pub enum Socks5Addr {
    V4(SocketAddr),
    V6(SocketAddr),
    Domain(String, u16),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Socks5Method {
    NoAuth = 0x00,
    Gssapi = 0x01,
    Password = 0x02,
    IANA(u8),
    Reserved(u8),
    NoAcceptable = 0xFF,
}

impl Socks5Method {
    pub const fn from_u8(value: u8) -> Self {
        match value {
            0x00 => Socks5Method::NoAuth,
            0x01 => Socks5Method::Gssapi,
            0x02 => Socks5Method::Password,
            0x03..=0x7F => Socks5Method::IANA(value),
            0x80..=0xFE => Socks5Method::Reserved(value),
            0xFF => Socks5Method::NoAcceptable,
        }
    }
    pub const fn into_u8(self) -> u8 {
        match self {
            Socks5Method::NoAuth => 0x00,
            Socks5Method::Gssapi => 0x01,
            Socks5Method::Password => 0x02,
            Socks5Method::IANA(value) => value,
            Socks5Method::Reserved(value) => value,
            Socks5Method::NoAcceptable => 0xFF,
        }
    }
}

impl From<Socks5Method> for u8 {
    fn from(method: Socks5Method) -> Self {
        method.into_u8()
    }
}

impl From<u8> for Socks5Method {
    fn from(value: u8) -> Self {
        Socks5Method::from_u8(value)
    }
}

const VERSION: u8 = 0x05;

pub trait Socks5Auth: Send + Sync + 'static {
    fn auth(
        &self,
        method: &mut dyn AsyncStream,
    ) -> Pin<Box<dyn Future<Output = io::Result<bool>> + Send>>;
}

#[derive(Clone)]
pub struct Socks5 {
    accepted_methods: Arc<Vec<(Socks5Method, Box<dyn Socks5Auth>)>>,
}

pub struct NoAuth;

impl Socks5Auth for NoAuth {
    fn auth(
        &self,
        _method: &mut dyn AsyncStream,
    ) -> Pin<Box<dyn Future<Output = io::Result<bool>> + Send>> {
        Box::pin(async { Ok(true) })
    }
}

impl Socks5 {
    pub fn no_auth() -> Self {
        Socks5 {
            accepted_methods: Arc::new(vec![(Socks5Method::NoAuth, Box::new(NoAuth))]),
        }
    }
}

async fn read_version<S>(stream: &mut S) -> io::Result<()>
where
    S: AsyncRead + Unpin,
{
    let version = stream.read_u8().await?;
    if version != VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid SOCKS version",
        ));
    }
    Ok(())
}

async fn write_response<S>(stream: &mut S, rep: u8, bind: SocketAddr) -> io::Result<()>
where
    S: AsyncWrite + Unpin,
{
    stream.write_all(&[VERSION, rep, RSV]).await?;
    match bind {
        SocketAddr::V4(addr) => {
            stream.write_all(&[ADDR_TYPE_IPV4]).await?;
            stream.write_all(&addr.ip().octets()).await?;
            stream.write_all(&addr.port().to_be_bytes()).await?;
        }
        SocketAddr::V6(addr) => {
            stream.write_all(&[ADDR_TYPE_IPV6]).await?;
            stream.write_all(&addr.ip().octets()).await?;
            stream.write_all(&addr.port().to_be_bytes()).await?;
        }
    }
    Ok(())
}

async fn read_request<S>(stream: &mut S) -> io::Result<Socks5Request>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    read_version(stream).await?;
    let cmd = stream.read_u8().await?;
    let _rsv = stream.read_u8().await?;
    let addr_type = stream.read_u8().await?;
    let addr = match addr_type {
        ADDR_TYPE_IPV4 => {
            let mut addr = [0u8; 4];
            stream.read_exact(&mut addr).await?;
            let port = stream.read_u16().await?;
            let addr = SocketAddr::new(addr.into(), port);
            Socks5Addr::V4(addr)
        }
        ADDR_TYPE_IPV6 => {
            let mut addr = [0u8; 16];
            stream.read_exact(&mut addr).await?;
            let port = stream.read_u16().await?;
            let addr = SocketAddr::new(addr.into(), port);
            Socks5Addr::V6(addr)
        }
        ADDR_TYPE_DOMAIN => {
            let domain_length = stream.read_u8().await? as usize;
            let mut domain = vec![0u8; domain_length];
            stream.read_exact(&mut domain).await?;
            let port = stream.read_u16().await?;
            let domain = String::from_utf8(domain)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid domain name"))?;
            Socks5Addr::Domain(domain, port)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid address type",
            ));
        }
    };
    let request = match cmd {
        CMD_CONNECT => Socks5Request::Connect(addr),
        CMD_BIND => Socks5Request::Bind(addr),
        CMD_UDP_ASSOCIATE => Socks5Request::UdpAssociate(addr),
        _ => {
            write_response(
                stream,
                REP_COMMAND_NOT_SUPPORTED,
                (Ipv4Addr::UNSPECIFIED, 0).into(),
            )
            .await?;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid command",
            ));
        }
    };
    Ok(request)
}

impl TcpService for Socks5 {
    async fn serve<S>(
        self: Arc<Self>,
        mut stream: S,
        peer: SocketAddr,
        ct: tokio_util::sync::CancellationToken,
    ) -> io::Result<()>
    where
        S: AsyncStream,
    {
        self.accept(&mut stream, peer, ct.child_token()).await
    }
}

impl Socks5 {
    pub async fn accept<S>(
        &self,
        stream: &mut S,
        _peer: SocketAddr,
        ct: tokio_util::sync::CancellationToken,
    ) -> io::Result<()>
    where
        S: AsyncStream,
    {
        read_version(stream).await?;
        let nmethods = stream.read_u8().await? as usize;
        if nmethods == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "No methods provided",
            ));
        }
        let mut methods = vec![0u8; nmethods];
        stream.read_exact(&mut methods).await?;
        let mut selected_method = Socks5Method::NoAcceptable;
        let mut selected_auth = None;
        for method in methods {
            if let Some((method, auth)) = self
                .accepted_methods
                .iter()
                .find(|(m, _)| m == &Socks5Method::from(method))
            {
                selected_method = *method;
                selected_auth = Some(auth.as_ref());
                break;
            }
        }

        let response = [VERSION, selected_method.into_u8()];
        stream.write_all(&response).await?;
        if let Some(auth) = selected_auth {
            if !auth.auth(stream).await? {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Authentication failed",
                ));
            }
        }

        let request = read_request(stream).await?;

        match request {
            Socks5Request::Connect(addr) => {
                let connect_result = match addr {
                    Socks5Addr::V4(socket_addr) => {
                        tokio::net::TcpStream::connect(socket_addr).await
                    }
                    Socks5Addr::V6(socket_addr) => {
                        tokio::net::TcpStream::connect(socket_addr).await
                    }
                    Socks5Addr::Domain(domain, port) => {
                        tokio::net::TcpStream::connect((domain, port)).await
                    }
                };
                let rep = match &connect_result {
                    Ok(_) => REP_SUCCESS,
                    Err(e) => match e.kind() {
                        io::ErrorKind::NetworkUnreachable => REP_NETWORK_UNREACHABLE,
                        io::ErrorKind::HostUnreachable => REP_HOST_UNREACHABLE,
                        io::ErrorKind::ConnectionRefused => REP_CONNECTION_REFUSED,
                        io::ErrorKind::TimedOut => REP_TTL_EXPIRED,
                        io::ErrorKind::AddrNotAvailable => REP_ADDRESS_TYPE_NOT_SUPPORTED,
                        io::ErrorKind::ConnectionReset => REP_CONNECTION_NOT_ALLOWED,
                        _ => REP_GENERAL_SOCKS_SERVER_FAILURE,
                    },
                };
                if let Ok(mut outbound) = connect_result {
                    let local_addr = outbound.local_addr()?;
                    write_response(stream, rep, local_addr).await?;
                    tokio::select! {
                        _ = ct.cancelled() => {
                            tracing::info!("Cancellation token triggered, shutting down server");
                            return Ok(());
                        }
                        result = tokio::io::copy_bidirectional(stream, &mut outbound) => {
                            tracing::info!("Outbound stream shutdown");
                            result?;
                        }
                    };
                } else {
                    write_response(stream, rep, (Ipv4Addr::UNSPECIFIED, 0).into()).await?;
                }
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unsupported command",
                ));
            }
        };
        Ok(())
    }
}

pub struct Socks5Provider;
impl TcpServiceProvider for Socks5Provider {
    const NAME: &'static str = "socks5";

    type Service = Socks5;

    type Error = Infallible;

    async fn construct(&self, _config: Option<String>) -> Result<Self::Service, Self::Error> {
        Ok(Socks5::no_auth())
    }
}
