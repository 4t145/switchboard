mod uds;

use std::{collections::HashMap, sync::Arc};

use switchboard_model::{
    control::{
        ControlCommandData, ControlSigner, ControllerMessage, KernelAuthResponse, KernelMessage,
        TakeOver, UpdateConfig,
    },
    kernel::{self, KernelInfoAndState},
};
use tokio::sync::RwLock;
use tracing::{Instrument, event};

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

#[derive(Debug, thiserror::Error)]
pub enum KernelConnectionError {
    #[error("connection error: {0}")]
    TransposeError(#[from] KernelConnectionTransposeError),
    #[error("unexpected message: {expected}, got: {actual:?}")]
    UnexpectedMessage {
        expected: &'static str,
        actual: KernelMessage,
    },
    #[error("connection closed")]
    ConnectionClosed,
    #[error("heartbeat timeout after {:.02} seconds", .after.as_secs_f64())]
    HeartbeatTimeout { after: tokio::time::Duration },
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
        tracing::info!("Connecting to kernel at {:?}", self);
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
                tracing::info!("Connected to kernel at {:?}", self);
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

pub struct KernelConnectionHandle {
    pub addr: KernelAddr,
    pub event_sender: tokio::sync::mpsc::Sender<KernelConnectionRequest>,
    pub kernel_state: Arc<RwLock<switchboard_model::kernel::KernelState>>,
    pub ct: tokio_util::sync::CancellationToken,
    pub handle: tokio::task::JoinHandle<Result<(), KernelConnectionError>>,
    pub info: switchboard_model::kernel::KernelInfo,
}

pub enum KernelConnectionRequest {
    SendCommand {
        command: ControlCommandData,
        ack: tokio::sync::oneshot::Sender<()>,
    },
}

impl KernelConnectionHandle {
    pub async fn get_info_and_state(&self) -> KernelInfoAndState {
        let state = self.kernel_state.read().await.clone();
        KernelInfoAndState {
            info: self.info.clone(),
            state,
        }
    }
    pub(crate) fn send_command(
        &self,
        command: ControlCommandData,
    ) -> impl Future<Output = Result<(), KernelConnectionError>> + Send + 'static {
        let event_sender = self.event_sender.clone();
        async move {
            let (ack_sender, ack_receiver) = tokio::sync::oneshot::channel();
            let request = KernelConnectionRequest::SendCommand {
                command,
                ack: ack_sender,
            };
            event_sender
                .send(request)
                .await
                .map_err(|_e| KernelConnectionError::ConnectionClosed)?;
            ack_receiver
                .await
                .map_err(|_e| KernelConnectionError::ConnectionClosed)?;
            Ok(())
        }
    }
    pub fn update_config(
        &self,
        new_config: switchboard_model::Config,
    ) -> impl Future<Output = Result<(), KernelConnectionError>> + Send + 'static {
        let command = ControlCommandData::UpdateConfig(UpdateConfig { config: new_config });
        self.send_command(command)
    }
    pub async fn get_state(&self) -> switchboard_model::kernel::KernelState {
        self.kernel_state.read().await.clone()
    }
    pub async fn close(self) -> Result<(), KernelConnectionError> {
        self.ct.cancel();
        let result = self.handle.await;
        match result {
            Ok(res) => res,
            Err(e) => {
                // this shouldn't happen because we didn't allow abort or panic happens
                tracing::error!(
                    "Kernel connection task join error: {} (this shouldn't happen)",
                    e
                );
                Ok(())
            }
        }
    }
    pub fn spawn(
        mut transpose: Transpose,
        addr: KernelAddr,
        info_and_state: kernel::KernelInfoAndState,
        context: &crate::ControllerContext,
    ) -> Self {
        let connect_config = &context.controller_config.kernel.connect;
        let heartbeat_interval = connect_config.heartbeat_interval;
        let ct = tokio_util::sync::CancellationToken::new();
        let ct_child = ct.child_token();
        let (event_sender, mut event_receiver) =
            tokio::sync::mpsc::channel::<KernelConnectionRequest>(
                connect_config.channel_buffer_size as usize,
            );
        enum Event {
            Request(KernelConnectionRequest),
            KernelMessage(KernelMessage),
        }
        let signer = ControlSigner::new(
            context.controller_config.kernel.psk.0.clone(),
            context.controller_config.info.name.clone(),
        );
        let kernel_state = Arc::new(RwLock::new(info_and_state.state));
        let span = tracing::info_span!(
            "kernel_connection_event_loop",
            kernel = %addr,
        );
        let task = {
            let mut pending_requests = HashMap::new();
            let kernel_state = kernel_state.clone();
            let addr = addr.clone();
            let context = context.clone();
            async move {
                let kernel_heartbeat_timeout_duration =
                    std::time::Duration::from_secs(heartbeat_interval as u64 * 3);
                let heartbeat_duration = std::time::Duration::from_secs(heartbeat_interval as u64);
                let mut heartbeat_timer = tokio::time::interval(heartbeat_duration);
                heartbeat_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);
                let mut kernel_heartbeat_timeout =
                    std::pin::pin!(tokio::time::sleep(kernel_heartbeat_timeout_duration));
                let addr = addr.clone();

                // connection event loop
                let loop_result = loop {
                    let event = tokio::select! {
                        // cancel on ct_child
                        _ = ct_child.cancelled() => {
                            tracing::info!("Kernel connection cancelled");
                            break Ok(());
                        }
                        // send heartbeat
                        _ = heartbeat_timer.tick() => {
                            transpose.send(ControllerMessage::HeartBeat).await?;
                            tracing::trace!("Sending heartbeat to kernel at {}", addr);
                            continue;
                        }
                        _ = &mut kernel_heartbeat_timeout => {
                            tracing::warn!("Kernel heartbeat timeout, closing connection to {}", addr);
                            break Err(KernelConnectionError::HeartbeatTimeout {
                                after: kernel_heartbeat_timeout_duration,
                            });
                        }
                        kernel_message = transpose.receive() => {
                            let message = kernel_message?;
                            Event::KernelMessage(message)
                        }
                        request = event_receiver.recv() => {
                            match request {
                                Some(request) => {
                                    Event::Request(request)
                                }
                                None => {
                                    tracing::info!("Kernel connection event channel closed");
                                    break Ok(());
                                }
                            }
                        }
                    };
                    match event {
                        Event::Request(request) => match request {
                            KernelConnectionRequest::SendCommand { command, ack } => {
                                let command = signer.sign_command(command);
                                pending_requests.insert(command.seq, ack);
                                transpose
                                    .send(ControllerMessage::ControlCommand(command))
                                    .await?;
                            }
                        },
                        Event::KernelMessage(kernel_message) => {
                            kernel_heartbeat_timeout.as_mut().reset(
                                tokio::time::Instant::now() + kernel_heartbeat_timeout_duration,
                            );
                            match kernel_message {
                                KernelMessage::HeartBeat(state) => {
                                    // update kernel state
                                    tracing::trace!(?state, "Received heartbeat from kernel");
                                    *kernel_state.write().await = state;
                                }
                                KernelMessage::ControlCommandAccepted(control_command_accepted) => {
                                    if let Some(ack) =
                                        pending_requests.remove(&control_command_accepted.seq)
                                    {
                                        let _ = ack.send(());
                                    } else {
                                        tracing::warn!(
                                            "No pending request for command seq: {}",
                                            control_command_accepted.seq
                                        );
                                    }
                                    tracing::trace!(
                                        "Kernel accepted control command: {:?}",
                                        control_command_accepted.seq
                                    );
                                }
                                KernelMessage::Disconnect => {
                                    tracing::info!(
                                        "Kernel requested disconnection, shutting down kernel connection task"
                                    );
                                    context
                                        .kernel_manager
                                        .write()
                                        .await
                                        .disconnect_kernel_without_close_connection(&addr);
                                    break Ok(());
                                }
                                _ => {
                                    tracing::warn!(
                                        "Unexpected kernel message: {:?}",
                                        kernel_message
                                    );
                                }
                            }
                        }
                    }
                };
                transpose.close().await?;
                loop_result
            }
        };
        let handle = tokio::spawn(
            async move {
                task.await
                    .inspect_err(|e| {
                        tracing::error!("Kernel connection task quit with error: {}", e);
                    })
                    .inspect(|_| {
                        tracing::info!("Kernel connection task ended");
                    })
            }
            .instrument(span),
        );
        KernelConnectionHandle {
            addr,
            event_sender,
            kernel_state,
            ct,
            handle,
            info: info_and_state.info,
        }
    }
}
