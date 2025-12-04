mod uds;

use anyhow::Context;
use switchboard_model::{
    control::{
        ControlCommand, ControlCommandData, ControllerMessage, KernelAuthResponse, KernelMessage,
        TakeOver,
    },
    kernel::KernelInfoAndState,
};

use crate::kernel::{KernelAddr, connection::uds::UdsTransposeConfig};
pub trait KernelTranspose: Send + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
    fn send(
        &mut self,
        message: ControllerMessage,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn receive(&mut self) -> impl Future<Output = Result<KernelMessage, Self::Error>> + Send;
    fn close(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub struct KernelConnectionHandle {
    pub addr: KernelAddr,
}

pub enum KernelConnectionEvent {
    SendCommand(ControlCommandData),
}

impl KernelConnectionHandle {}

#[derive(Debug, thiserror::Error)]
pub enum KernelConnectionError {
    #[error("connection error: {0}")]
    TransposeError(#[from] KernelConnectionTransposeError),
    #[error("unexpected message: {expected}, got: {actual:?}")]
    UnexpectedMessage {
        expected: &'static str,
        actual: KernelMessage,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum KernelConnectionTransposeError {
    #[error("uds connection error: {0}")]
    UdsConnectionError(#[from] uds::UdsTransposeError),
}

pub enum Transpose {
    Uds(Box<uds::UdsTranspose>),
}

impl KernelTranspose for Transpose {
    type Error = KernelConnectionTransposeError;
    async fn send(
        &mut self,
        message: switchboard_model::control::ControllerMessage,
    ) -> Result<(), Self::Error> {
        match self {
            Transpose::Uds(conn) => conn
                .send(message)
                .await
                .map_err(KernelConnectionTransposeError::from),
        }
    }
    async fn receive(&mut self) -> Result<switchboard_model::control::KernelMessage, Self::Error> {
        match self {
            Transpose::Uds(conn) => conn
                .receive()
                .await
                .map_err(KernelConnectionTransposeError::from),
        }
    }
    async fn close(self) -> Result<(), Self::Error> {
        match self {
            Transpose::Uds(conn) => conn
                .close()
                .await
                .map_err(KernelConnectionTransposeError::from),
        }
    }
}

impl KernelAddr {
    pub async fn connect(
        &self,
        config: crate::config::KernelConfig,
    ) -> Result<Transpose, KernelConnectionError> {
        match self {
            KernelAddr::Uds(path) => {
                let connection = uds::UdsTranspose::connect(UdsTransposeConfig {
                    path: path.clone(),
                    max_frame_size: config.discovery.uds.max_frame_size,
                })
                .await
                .map_err(uds::UdsTransposeError::from)
                .map_err(KernelConnectionTransposeError::from)
                .map_err(KernelConnectionError::from)?;
                Ok(Transpose::Uds(Box::new(connection)))
            }
            KernelAddr::Tcp(_url) => {
                unimplemented!()
            }
        }
    }
}

impl Transpose {
    pub async fn take_over(
        &mut self,
        context: &crate::ControllerContext,
    ) -> Result<KernelInfoAndState, KernelConnectionError> {
        self.send(ControllerMessage::TakeOver(TakeOver {
            controller_info: context.controller_config.info.clone(),
        }))
        .await?;
        let maybe_auth = self.receive().await?;
        let auth = if let KernelMessage::Auth(auth) = maybe_auth {
            auth
        } else {
            return Err(KernelConnectionError::UnexpectedMessage {
                expected: "KernelInfo",
                actual: maybe_auth,
            });
        };
        let kernel_info = auth.kernel_info.clone();
        let controller_message = ControllerMessage::AuthResponse(KernelAuthResponse::sign(
            &auth,
            &context.controller_config.kernel.psk,
        ));
        self.send(controller_message).await?;
        // wait for heart beat
        let maybe_heartbeat = self.receive().await?;
        let kernel_state = if let KernelMessage::HeartBeat(state) = maybe_heartbeat {
            state
        } else {
            return Err(KernelConnectionError::UnexpectedMessage {
                expected: "HeartBeat",
                actual: maybe_heartbeat,
            });
        };
        Ok(KernelInfoAndState {
            info: kernel_info,
            state: kernel_state,
        })
    }
}
