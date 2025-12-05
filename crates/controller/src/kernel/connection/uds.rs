use std::path::PathBuf;

use switchboard_model::{
    control::{ControllerMessage, KernelMessage},
    kernel::UDS_DEFAULT_PATH,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

pub struct UdsTransposeConfig {
    pub path: PathBuf,
    pub max_frame_size: u32,
}

impl Default for UdsTransposeConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from(UDS_DEFAULT_PATH),
            max_frame_size: 1 << 22,
        }
    }
}

pub struct UdsTranspose {
    pub stream: UnixStream,
    pub addr: tokio::net::unix::SocketAddr,
    pub config: UdsTransposeConfig,
    pub write_buffer: Vec<u8>,
    pub read_buffer: Vec<u8>,
}

impl UdsTranspose {
    pub async fn connect(config: UdsTransposeConfig) -> Result<Self, UdsEstablishTransposeError> {
        let stream = UnixStream::connect(&config.path).await?;
        let addr = stream.peer_addr()?;
        Ok(Self::new(stream, addr, config))
    }
    pub fn new(
        stream: UnixStream,
        addr: tokio::net::unix::SocketAddr,
        config: UdsTransposeConfig,
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
    async fn receive_next(&mut self) -> Result<KernelMessage, UdsTransposeReadError> {
        let size = self.stream.read_u32().await?;
        if size > self.config.max_frame_size {
            return Err(UdsTransposeReadError::FrameSizeExceeded {
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
        message: &ControllerMessage,
    ) -> Result<(), UdsTransposeWriteError> {
        self.write_buffer.clear();
        let size = bincode::encode_into_std_write(
            message,
            &mut self.write_buffer,
            bincode::config::standard(),
        )? as u32;
        if size > self.config.max_frame_size {
            return Err(UdsTransposeWriteError::FrameSizeExceeded {
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

impl super::KernelTranspose for UdsTranspose {
    type Error = UdsTransposeError;
    async fn send(
        &mut self,
        message: switchboard_model::control::ControllerMessage,
    ) -> Result<(), Self::Error> {
        self.send_and_flush(&message).await?;
        Ok(())
    }
    async fn receive(&mut self) -> Result<switchboard_model::control::KernelMessage, Self::Error> {
        let message = self.receive_next().await?;
        Ok(message)
    }
    async fn close(mut self) -> Result<(), Self::Error> {
        self.stream
            .shutdown()
            .await
            .map_err(UdsTransposeError::Close)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UdsTransposeError {
    #[error("read error: {0}")]
    Read(#[from] UdsTransposeReadError),
    #[error("write error: {0}")]
    Write(#[from] UdsTransposeWriteError),
    #[error("connect error: {0}")]
    Connect(#[from] UdsEstablishTransposeError),
    #[error("close error: {0}")]
    Close(#[source] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum UdsEstablishTransposeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
#[derive(Debug, thiserror::Error)]
pub enum UdsTransposeReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode decode error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded { max_size: u32, actual_size: u32 },
}

#[derive(Debug, thiserror::Error)]
pub enum UdsTransposeWriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode encode error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("frame size exceeds maximum: max: {max_size}, actual: {actual_size}")]
    FrameSizeExceeded { max_size: u32, actual_size: u32 },
}
