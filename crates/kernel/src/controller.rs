pub mod listener;

use crate::{KernelContext, controller::listener::TransportError};
use serde::{Deserialize, Serialize};
use switchboard_model::{
    bytes::Base64Bytes, control::*, kernel::KernelState, protocol::DEFAULT_HEARTBEAT_INTERVAL_SECS,
};
use tracing::Instrument;

pub trait ControllerTransport: Send + 'static {
    type Error: Into<TransportError> + Send + Sync + 'static;
    fn peer(&self) -> impl std::fmt::Display + Send + Sync + 'static;
    fn send(
        &mut self,
        message: KernelMessage,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn receive(&mut self) -> impl Future<Output = Result<ControllerMessage, Self::Error>> + Send;
    fn close(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("expected take over message, but got: {actual:?}")]
    ExpectTakeOver { actual: ControllerMessage },
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("controller transport error: {source}, when {context}")]
    Transport {
        context: String,
        #[source]
        source: TransportError,
    },
    #[error("authentication error: {0}")]
    AuthError(String),
    #[error("controller timeout after {} seconds", after.as_secs())]
    Timeout { after: std::time::Duration },
    #[error("controller already shut down")]
    AlreadyShutDown,
}

impl ConnectError {
    pub fn when<
        E: Into<TransportError> + Send + Sync + 'static,
        C: std::fmt::Display + Send + Sync + 'static,
    >(
        context: C,
    ) -> impl FnOnce(E) -> Self {
        move |e| ConnectError::Transport {
            context: context.to_string(),
            source: e.into(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllerConfig {
    pub psk: Base64Bytes,
    #[serde(default)]
    pub connect: ConnectConfig,
    pub listen: Option<listener::ListenerConfig>,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            psk: Base64Bytes(rand::random::<[u8; 16]>().to_vec()),
            connect: ConnectConfig::default(),
            listen: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectConfig {
    /// Heartbeat interval in seconds, default to 5 seconds
    #[serde(default)]
    pub heartbeat_interval: u32,
    #[serde(default)]
    /// Channel capacity for sending messages to controller, default to be 32
    pub channel_capacity: u32,
}

impl Default for ConnectConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL_SECS,
            channel_capacity: 32,
        }
    }
}

pub async fn report_been_took_over<C: ControllerTransport>(
    mut controller_port: C,
    take_over: TakeOver,
) -> Result<(), ConnectError> {
    controller_port
        .send(KernelMessage::BeenTookOver(BeenTookOver {
            new_controller_info: take_over.controller_info.clone(),
        }))
        .await
        .map_err(ConnectError::when("sending been took over message"))?;
    controller_port
        .close()
        .await
        .map_err(ConnectError::when("closing controller port"))?;
    Ok(())
}

pub struct ControllerHandle {
    message_sender: tokio::sync::mpsc::Sender<ControllerEvent>,
    ct: tokio_util::sync::CancellationToken,
    task_handle: tokio::task::JoinHandle<Result<(), ConnectError>>,
}

pub(crate) enum ControllerEvent {
    TakeOver(TakeOver),
    UpdateState(KernelState),
    HeartBeat,
    ReceiveMessage(ControllerMessage),
    Cancellation,
}

impl ControllerHandle {
    pub async fn shutdown(self) {
        self.ct.cancel();
        let _ = self.task_handle.await.inspect_err(|err| {
            tracing::error!(
                "Controller connection task ended with error during shutdown: {}",
                err
            );
        });
    }

    pub async fn spawn<C: ControllerTransport>(
        mut controller_transport: C,
        context: KernelContext,
    ) -> Self {
        let (message_sender, mut message_receiver) = tokio::sync::mpsc::channel::<ControllerEvent>(
            context.kernel_config.controller.connect.channel_capacity as usize,
        );
        let ct = tokio_util::sync::CancellationToken::new();
        let ct_child = ct.child_token();
        let mut hb_interval = tokio::time::interval(std::time::Duration::from_secs(
            context.kernel_config.controller.connect.heartbeat_interval as u64,
        ));
        let controller_heartbeat_timeout_duration = std::time::Duration::from_secs(
            (context.kernel_config.controller.connect.heartbeat_interval as u64) * 3,
        );
        let verifier = switchboard_model::control::ControlVerifier {
            sign_key: context.kernel_config.controller.psk.0.clone(),
        };
        let peer = controller_transport.peer();
        let task = async move {
            let mut controller_heartbeat_timeout =
                std::pin::pin!(tokio::time::sleep(controller_heartbeat_timeout_duration));
            tracing::debug!("spawning controller connection event loop");
            let loop_result = loop {
                let next_event = tokio::select! {
                    // task cancelled
                    _ = ct_child.cancelled() => {
                        tracing::info!("controller connection task is cancelled, shutting down");
                        ControllerEvent::Cancellation
                    }
                    // heartbeat interval
                    _ = hb_interval.tick() => {
                        ControllerEvent::HeartBeat
                    }
                    _ =  &mut controller_heartbeat_timeout => {
                        tracing::warn!("controller heartbeat timeout after {} seconds, no heartbeat received in time", controller_heartbeat_timeout_duration.as_secs());
                        break Err(ConnectError::Timeout {
                            after: controller_heartbeat_timeout_duration,
                        });
                    }
                    controller_message = controller_transport.receive() => {
                        controller_heartbeat_timeout.as_mut().set(tokio::time::sleep(controller_heartbeat_timeout_duration));
                        let message = controller_message
                            .map_err(ConnectError::when("receiving controller message"))?;
                        ControllerEvent::ReceiveMessage(message)
                    }
                    maybe_message = message_receiver.recv() => {
                        let Some(event) = maybe_message else {
                            tracing::info!("message receiver dropped, shutting down controller connection task");
                            break Ok(());
                        };
                        event
                    }
                };
                match next_event {
                    ControllerEvent::TakeOver(take_over) => {
                        controller_transport
                            .send(KernelMessage::BeenTookOver(BeenTookOver {
                                new_controller_info: take_over.controller_info.clone(),
                            }))
                            .await
                            .map_err(ConnectError::when("sending take over message"))?;
                        tracing::info!(
                            "been took over message sent, shutting down controller connection task"
                        );
                        break Ok(());
                    }
                    ControllerEvent::UpdateState(kernel_state) => {
                        controller_transport
                            .send(KernelMessage::HeartBeat(kernel_state))
                            .await
                            .map_err(ConnectError::when("sending take over message"))?;
                        hb_interval.reset();
                    }
                    ControllerEvent::HeartBeat => {
                        let kernel_state = context.get_state().await;
                        controller_transport
                            .send(KernelMessage::HeartBeat(kernel_state))
                            .await
                            .map_err(ConnectError::when("sending heartbeat message"))?;
                    }
                    ControllerEvent::ReceiveMessage(controller_message) => match controller_message
                    {
                        ControllerMessage::HeartBeat => {
                            // do nothing, we already reset the timeout
                        }
                        ControllerMessage::ControlCommand(cmd) => {
                            let seq = cmd.seq;

                            let verify_result = verifier.verify_command(&cmd);
                            if let Err(e) = verify_result {
                                tracing::warn!(
                                    "fail to verify command: {} from {}",
                                    e,
                                    cmd.signer_name
                                );
                                continue;
                            }
                            context.handle_control_command(cmd).await;
                            controller_transport
                                .send(KernelMessage::ControlCommandAccepted(
                                    ControlCommandAccepted { seq },
                                ))
                                .await
                                .map_err(ConnectError::when(
                                    "sending control command accepted message",
                                ))?;
                        }
                        ControllerMessage::Disconnect => {
                            tracing::info!("controller requested disconnection, shutting down controller connection task");
                            break Ok(());
                        },
                        _ => {
                            tracing::warn!(
                                "received unexpected controller message in the controller connection event loop: {:?}",
                                controller_message
                            );
                        }
                    },
                    ControllerEvent::Cancellation => {
                        tracing::info!(
                            "received cancellation event, shutting down controller connection task"
                        );
                        controller_transport
                            .send(KernelMessage::Disconnect)
                            .await
                            .map_err(ConnectError::when(
                                "sending control command accepted message",
                            ))?;
                        break Ok(());
                    }
                };
            };
            controller_transport
                .close()
                .await
                .map_err(ConnectError::when("closing"))?;
            return loop_result;
        };
        let task_handle = tokio::spawn(
            async {
                task.await
                    .inspect_err(|e| {
                        tracing::error!("Controller connection task ended with error: {}", e);
                    })
                    .inspect(|_| {
                        tracing::info!("Controller connection task ended normally");
                    })
            }
            .instrument(tracing::info_span!(
                "controller_connection_event_loop",
                peer = %peer,
            )),
        );
        ControllerHandle {
            message_sender,
            ct,
            task_handle,
        }
    }
    pub async fn take_over(self, take_over: TakeOver) -> Result<(), ConnectError> {
        if let Err(e) = self
            .message_sender
            .send(ControllerEvent::TakeOver(take_over))
            .await
        {
            tracing::warn!(
                "failed to send take over event to controller connection task: {}",
                e
            );
        }
        self.ct.cancel();
        self.task_handle.await.ok();
        Ok(())
    }
    pub async fn update_state(&self, kernel_state: KernelState) -> Result<(), ConnectError> {
        self.message_sender
            .send(ControllerEvent::UpdateState(kernel_state))
            .await
            .map_err(|_| ConnectError::AlreadyShutDown)?;
        Ok(())
    }
}

impl KernelContext {
    pub async fn handle_control_command(&self, cmd: ControlCommand) {
        match cmd.data {
            ControlCommandData::Quit => {
                tracing::info!("received quit command from controller");
                self.shutdown().await;
            }
            ControlCommandData::UpdateConfig(update_config) => {
                tracing::info!("received update config command from controller");
                if let Err(e) = self.update_config(update_config.config).await {
                    tracing::error!("failed to update config: {}", e);
                }
            }
        }
    }
    pub async fn start_up_connection<C: ControllerTransport>(
        &self,
        controller_port: &mut C,
        config: &ControllerConfig,
    ) -> Result<TakeOver, ConnectError> {
        // expect take over message
        let maybe_takeover = controller_port
            .receive()
            .await
            .map_err(ConnectError::when("receiving take over message"))?;
        let take_over = match maybe_takeover {
            ControllerMessage::TakeOver(take_over) => take_over,
            other => {
                return Err(ProtocolError::ExpectTakeOver { actual: other }.into());
            }
        };
        let random_bytes: [u8; 16] = rand::random();
        let auth = KernelAuth {
            random_bytes: random_bytes.to_vec(),
            kernel_info: self.kernel_config.info.clone(),
        };
        controller_port
            .send(KernelMessage::Auth(auth.clone()))
            .await
            .map_err(ConnectError::when("sending auth message"))?;
        let maybe_auth_response = controller_port
            .receive()
            .await
            .map_err(ConnectError::when("receiving auth response"))?;
        let auth_response = match maybe_auth_response {
            ControllerMessage::AuthResponse(r) => r,
            other => {
                return Err(ProtocolError::ExpectTakeOver { actual: other }.into());
            }
        };
        // verify auth response
        auth_response
            .verify(&auth, &config.psk)
            .map_err(|e| ConnectError::AuthError(e.to_string()))?;
        // send a heartbeat immediately after authentication
        let state = self.get_state().await;
        controller_port
            .send(KernelMessage::HeartBeat(state))
            .await
            .map_err(ConnectError::when("sending initial heartbeat message"))?;
        Ok(take_over)
    }
    pub async fn spawn_controller_connection_event_loop<C: ControllerTransport>(
        &self,
        mut controller_connection: C,
    ) -> Result<(), ConnectError> {
        let peer = controller_connection.peer();
        tracing::info!("accepted new controller connection from {}", peer);
        let context = self.clone();
        let take_over = context
            .start_up_connection(
                &mut controller_connection,
                &context.kernel_config.controller,
            )
            .await?;
        tracing::debug!(
            "successful startup handshake with controller at {}, take over info: {:?}",
            peer,
            take_over
        );
        if let Some(handle) = context.controller_handle.write().await.take() {
            tokio::spawn(async move {
                if let Err(e) = handle.take_over(take_over).await {
                    tracing::error!("failed to take over existing controller connection: {}", e);
                }
            });
        }
        let new_controller_handle =
            ControllerHandle::spawn(controller_connection, context.clone()).await;
        *context.controller_handle.write().await = Some(new_controller_handle);
        Ok(())
    }
    pub async fn shutdown_controller(&self) {
        if let Some(handle) = self.controller_handle.write().await.take() {
            tracing::info!("shutting down controller connection");
            handle.shutdown().await;
        }
    }
}
