use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use switchboard_model::control::{ControllerMessage, KernelMessage};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UdsListenerConfig {
    #[serde(default = "default_path")]
    pub path: PathBuf,
    #[serde(default = "default_max_frame_size")]
    pub max_frame_size: u32,
}

fn default_path() -> PathBuf {
    PathBuf::from("/var/run/switchboard/default.sock")
}

const fn default_max_frame_size() -> u32 {
    1 << 22
}
pub struct UdsListener {
    pub config: UdsListenerConfig,
    pub listener: tokio::net::UnixListener,
}

impl UdsListener {
    pub async fn new(config: UdsListenerConfig) -> std::io::Result<Self> {
        // check if the socket dir exists
        if let Some(parent) = config.path.parent()
            && !parent.exists()
        {
            tokio::fs::create_dir_all(parent).await?;
        }
        let listener = tokio::net::UnixListener::bind(&config.path)?;
        Ok(Self { config, listener })
    }
    pub async fn accept(&self) -> std::io::Result<UdsConnection> {
        let (stream, peer) = self.listener.accept().await?;
        Ok(UdsConnection::new(stream, peer, self.config.clone()))
    }
}

pub struct UdsConnection {
    pub stream: UnixStream,
    pub addr: tokio::net::unix::SocketAddr,
    pub config: UdsListenerConfig,
    pub write_buffer: Vec<u8>,
    pub read_buffer: Vec<u8>,
}

impl UdsConnection {
    pub fn new(
        stream: UnixStream,
        addr: tokio::net::unix::SocketAddr,
        config: UdsListenerConfig,
    ) -> Self {
        const INITIAL_BUFFER_SIZE: usize = 1 << 10;
        Self {
            stream,
            addr,
            config,
            write_buffer: Vec::with_capacity(INITIAL_BUFFER_SIZE),
            read_buffer: Vec::with_capacity(INITIAL_BUFFER_SIZE),
        }
    }
    async fn receive_next(&mut self) -> Result<ControllerMessage, UdsConnectionReadError> {
        let size = self.stream.read_u32().await?;
        if size > self.config.max_frame_size {
            return Err(UdsConnectionReadError::FrameSizeExceeded {
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
    ) -> Result<(), UdsConnectionWriteError> {
        self.write_buffer.clear();
        bincode::encode_into_slice(message, &mut self.write_buffer, bincode::config::standard())?;
        let size = self.write_buffer.len() as u32;
        if size > self.config.max_frame_size {
            return Err(UdsConnectionWriteError::FrameSizeExceeded {
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

impl crate::controller::ControllerConnection for UdsConnection {
    type Error = UdsConnectionError;
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
pub enum UdsConnectionError {
    #[error("read error: {0}")]
    Read(#[from] UdsConnectionReadError),
    #[error("write error: {0}")]
    Write(#[from] UdsConnectionWriteError),
}

#[derive(Debug, thiserror::Error)]
pub enum UdsConnectionReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode decode error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded { max_size: u32, actual_size: u32 },
}

#[derive(Debug, thiserror::Error)]
pub enum UdsConnectionWriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode encode error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded { max_size: u32, actual_size: u32 },
}
