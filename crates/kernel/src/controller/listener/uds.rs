use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use switchboard_model::{
    control::{ControllerMessage, KernelMessage},
    kernel::UDS_DEFAULT_PATH,
};
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
    PathBuf::from(UDS_DEFAULT_PATH)
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
        } else {
            // if the socket file exists, remove it
            if config.path.exists() {
                tokio::fs::remove_file(&config.path).await?;
            }
        }
        let listener = tokio::net::UnixListener::bind(&config.path)?;
        Ok(Self { config, listener })
    }
    pub async fn accept(&self) -> std::io::Result<UdsTransport> {
        let (stream, peer) = self.listener.accept().await?;
        Ok(UdsTransport::new(stream, peer, self.config.clone()))
    }
}

pub struct UdsTransport {
    pub stream: UnixStream,
    pub addr: tokio::net::unix::SocketAddr,
    pub config: UdsListenerConfig,
    pub write_buffer: Vec<u8>,
    pub read_buffer: Vec<u8>,
}

impl UdsTransport {
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
    async fn receive_next(&mut self) -> Result<ControllerMessage, UdsTransportReadError> {
        let size = self.stream.read_u32().await?;
        if size > self.config.max_frame_size {
            return Err(UdsTransportReadError::FrameSizeExceeded {
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
    ) -> Result<(), UdsTransportWriteError> {
        self.write_buffer.clear();
        let size = bincode::encode_into_std_write(
            message,
            &mut self.write_buffer,
            bincode::config::standard(),
        )? as u32;
        if size > self.config.max_frame_size {
            return Err(UdsTransportWriteError::FrameSizeExceeded {
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

impl crate::controller::ControllerTransport for UdsTransport {
    type Error = UdsTransportError;
    fn peer(&self) -> impl std::fmt::Display + Send + Sync + 'static {
        use base64::prelude::*;
        self.addr
            .as_pathname()
            .and_then(|p| p.to_str())
            .map(|p| p.to_owned())
            .or(self
                .addr
                .as_abstract_name()
                .map(|n| BASE64_STANDARD.encode(n)))
            .unwrap_or("unnamed".to_owned())
    }
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
    async fn close(mut self) -> Result<(), Self::Error> {
        // delete the underlying socket file
        self.stream.shutdown().await?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UdsTransportError {
    #[error("read error: {0}")]
    Read(#[from] UdsTransportReadError),
    #[error("write error: {0}")]
    Write(#[from] UdsTransportWriteError),
    #[error("close error: {0}")]
    Close(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum UdsTransportReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode decode error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded { max_size: u32, actual_size: u32 },
}

#[derive(Debug, thiserror::Error)]
pub enum UdsTransportWriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode encode error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded { max_size: u32, actual_size: u32 },
}
