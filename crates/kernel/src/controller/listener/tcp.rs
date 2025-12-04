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

pub struct TcpListener {
    pub config: TcpListenerConfig,
    pub listener: tokio::net::TcpListener,
}

impl TcpListener {
    pub async fn new(config: TcpListenerConfig) -> std::io::Result<Self> {
        let listener = tokio::net::TcpListener::bind((config.host, config.port)).await?;
        Ok(Self { config, listener })
    }
    pub async fn accept(&self) -> std::io::Result<TcpConnection> {
        let (stream, peer) = self.listener.accept().await?;
        Ok(TcpConnection::new(stream, peer, self.config.clone()))
    }
}

pub struct TcpConnection {
    pub stream: TcpStream,
    pub addr: std::net::SocketAddr,
    pub config: TcpListenerConfig,
    pub write_buffer: Vec<u8>,
    pub read_buffer: Vec<u8>,
}

impl TcpConnection {
    pub fn new(stream: TcpStream, addr: std::net::SocketAddr, config: TcpListenerConfig) -> Self {
        const INITIAL_BUFFER_SIZE: usize = 1 << 10;
        Self {
            stream,
            addr,
            config,
            write_buffer: Vec::with_capacity(INITIAL_BUFFER_SIZE),
            read_buffer: Vec::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }
    async fn receive_next(&mut self) -> Result<ControllerMessage, TcpConnectionReadError> {
        let size = self.stream.read_u32().await?;
        if size > self.config.max_frame_size {
            return Err(TcpConnectionReadError::FrameSizeExceeded {
                max_size: self.config.max_frame_size,
                actual_size: size,
            });
        }
        if self.read_buffer.len() < size as usize {
            self.read_buffer.resize(size as usize, 0);
        }
        self.stream
            .read_exact(&mut self.read_buffer[..size as usize])
            .await?;
        let (message, _) = bincode::decode_from_slice(
            &self.read_buffer[..size as usize],
            bincode::config::standard(),
        )?;
        // self.read_buffer.clear();
        Ok(message)
    }
    async fn send_and_flush(
        &mut self,
        message: &KernelMessage,
    ) -> Result<(), TcpConnectionWriteError> {
        self.write_buffer.clear();
        bincode::encode_into_slice(message, &mut self.write_buffer, bincode::config::standard())?;
        let size = self.write_buffer.len() as u32;
        if size > self.config.max_frame_size {
            return Err(TcpConnectionWriteError::FrameSizeExceeded {
                max_size: self.config.max_frame_size,
                actual_size: size,
            });
        }
        self.stream.write_u32(size).await?;
        self.stream.write_all(&self.write_buffer).await?;
        self.stream.flush().await?;
        Ok(())
    }
}

impl crate::controller::ControllerConnection for TcpConnection {
    type Error = TcpConnectionError;
    async fn send(
        &mut self,
        message: switchboard_model::control::KernelMessage,
    ) -> Result<(), Self::Error> {
        self.send_and_flush(&message).await?;
        Ok(())
    }
    async fn receive(
        &mut self,
    ) -> Result<switchboard_model::control::ControllerMessage, Self::Error> {
        let message = self.receive_next().await?;
        Ok(message)
    }
    async fn close(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TcpConnectionError {
    #[error("read error: {0}")]
    Read(#[from] TcpConnectionReadError),
    #[error("write error: {0}")]
    Write(#[from] TcpConnectionWriteError),
}

#[derive(Debug, thiserror::Error)]
pub enum TcpConnectionReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode decode error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded { max_size: u32, actual_size: u32 },
}

#[derive(Debug, thiserror::Error)]
pub enum TcpConnectionWriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode encode error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded { max_size: u32, actual_size: u32 },
}
