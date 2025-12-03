use std::path::PathBuf;

use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::error;
use switchboard_model::control::ControllerMessage;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{UnixStream, unix::OwnedReadHalf}};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UdsListenerConfig {
    pub path: PathBuf,
    pub max_frame_size: u32,
}

pub async fn uds_listener(config: UdsListenerConfig) -> std::io::Result<()> {
    let listener = tokio::net::UnixListener::bind(config.path)?;
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (read, write) = stream.into_split();
            let mut buf = vec![0u8; 1 << 10];
            loop {
                let size = read.read_u32().await?;
                
                if size > config.max_frame_size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "frame size exceeds maximum",
                    ));
                }
                if buf.len() < size as usize {
                    buf.resize(size as usize, 0);
                }
                read.read_exact(&mut buf[..size as usize]).await?;
                let message = bincode::decode_from_slice(&buf[..size as usize], bincode::config::standard())?;
                
            }
            // Handle the connection
            let _ = stream;
        });
    }


}

pub struct UdsConnectionRx {
    pub rx: OwnedReadHalf,
    pub config: UdsListenerConfig,
    pub read_buffer: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum UdsConnectionReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode decode error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded {
        max_size: u32,
        actual_size: u32,
    },
}

impl UdsConnectionRx {
    pub async fn receive_next(&mut self) -> Result<ControllerMessage, UdsConnectionReadError> {
        let size = self.rx.read_u32().await?;
        if size > self.config.max_frame_size {
            return Err(UdsConnectionReadError::FrameSizeExceeded {
                max_size: self.config.max_frame_size,
                actual_size: size,
            });
        }
        if self.read_buffer.len() < size as usize {
            self.read_buffer.resize(size as usize, 0);
        }
        self.rx.read_exact(&mut self.read_buffer[..size as usize]).await?;
        let (message, _) = bincode::decode_from_slice(&self.read_buffer[..size as usize], bincode::config::standard())?;
        // self.read_buffer.clear();
        Ok(message)
    }
    pub fn new(rx: OwnedReadHalf, config: UdsListenerConfig) -> Self {
        const INITIAL_BUFFER_SIZE: usize = 1 << 10;
        Self {
            rx,
            config,
            read_buffer: vec![0u8; INITIAL_BUFFER_SIZE],
        }
    }
}

pub struct UdsConnectionTx {
    pub tx: tokio::net::unix::OwnedWriteHalf,
    pub config: UdsListenerConfig,
    pub write_buffer: Vec<u8>,
}
#[derive(Debug, thiserror::Error)]
pub enum UdsConnectionWriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode encode error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded {
        max_size: u32,
        actual_size: u32,
    },
}

impl UdsConnectionTx {
    pub async fn send(&mut self, message: &ControllerMessage) -> Result<(), UdsConnectionWriteError> {
        self.write_buffer.clear();
        bincode::encode_into_slice(
            message,
            &mut self.write_buffer,
            bincode::config::standard(),
        )?;
        let size = self.write_buffer.len() as u32;
        if size > self.config.max_frame_size {
            return Err(UdsConnectionWriteError::FrameSizeExceeded {
                max_size: self.config.max_frame_size,
                actual_size: size,
            });
        }
        self.tx.write_u32(size).await?;
        self.tx.write_all(&self.write_buffer).await?;
        Ok(())
    }
    pub fn new(tx: tokio::net::unix::OwnedWriteHalf, config: UdsListenerConfig) -> Self {
        Self { tx, config, write_buffer: Vec::new() }
    }
}