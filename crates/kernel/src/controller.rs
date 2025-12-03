pub mod listener;
use std::{any, ops::Deref, sync::Arc};

use deno_core::serde::{Deserialize, Serialize};
use futures::{Sink, Stream};
use switchboard_model::control::*;

use crate::KernelContext;

pub trait ControllerPort: Send + 'static {
    type Error: std::error::Error + Send + Sync + 'static;
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
    #[error("controller port error: {0}")]
    Port(#[from] anyhow::Error),
    #[error("authentication error: {0}")]
    AuthError(String),
}

impl ConnectError {
    pub fn when<
        E: std::error::Error + Send + Sync + 'static,
        C: std::fmt::Display + Send + Sync + 'static,
    >(
        context: C,
    ) -> impl FnOnce(E) -> Self {
        move |e| ConnectError::Port(anyhow::Error::new(e).context(context))
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllerConfig {
    pub connect: ConnectConfig,
    pub listen: Option<listener::ListenerConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectConfig {
    pub psk: Vec<u8>,
}
pub async fn start_up_connection<C: ControllerPort>(
    controller_port: &mut C,
    config: &ConnectConfig,
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
    // send
    Ok(take_over)
}

pub async fn report_been_took_over<C: ControllerPort>(
    mut controller_port: C,
    take_over: TakeOver,
) -> Result<(), ConnectError> {
    controller_port
        .send(KernelMessage::BeenTookOver(BeenTookOver {
            new_controller_name: take_over.controller_name,
        }))
        .await
        .map_err(ConnectError::when("sending been took over message"))?;
    controller_port
        .close()
        .await
        .map_err(ConnectError::when("closing controller port"))?;
    Ok(())
}

pub struct ControllerConnection {

}
impl ControllerConnection {
    pub fn spawn<C: ControllerPort>(mut controller_port: C, ) {
        tokio::spawn(async move {
            
        });
    }
}

pub struct ControllerHandle {
    pub message_sender: tokio::sync::mpsc::Sender<KernelMessage>,
    pub ct: tokio_util::sync::CancellationToken,
    task_handle: tokio::task::JoinHandle<Result<(), ConnectError>>,
}
impl ControllerHandle {
    pub async fn spawn<C: ControllerPort>(
        mut controller_port: C,
        ct: tokio_util::sync::CancellationToken,
    ) -> Self {
        // todo:
        // 1. 双向心跳机制
        // 2. 接收controller消息并且执行对应动作
        let (message_sender, mut message_receiver) =
            tokio::sync::mpsc::channel::<KernelMessage>(16);
        let ct_child = ct.child_token();
        let task_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = ct_child.cancelled() => {
                        tracing::info!("controller connection task is cancelled, shutting down");
                        break;
                    }
                    maybe_message = message_receiver.recv() => {
                        match maybe_message {
                            Some(message) => {
                                let is_been_took_over = message.is_been_took_over();
                                controller_port.send(message).await.map_err(ConnectError::when("sending normal message"))?;
                                if is_been_took_over {
                                    tracing::info!("been took over message sent, shutting down controller connection task");
                                    controller_port.close().await.map_err(ConnectError::when("closing after been took over"))?;
                                    break;
                                }
                            }
                            None => {
                                tracing::info!("message sender dropped, shutting down controller connection task");
                                break;
                            }
                        }
                    }
                }
            }
            Ok(())
        });
        ControllerHandle {
            message_sender,
            ct,
            task_handle,
        }
    }
    pub async fn take_over(
        self,
        take_over: TakeOver,
    ) -> Result<(), ConnectError> {
        self.message_sender
            .send(KernelMessage::BeenTookOver(BeenTookOver {
                new_controller_name: take_over.controller_name,
            })).await
            .map_err(|e| {
                ConnectError::Port(anyhow::Error::new(e).context("sending been took over message"))
            })?;
        self.ct.cancel();
        self.task_handle.await.ok();
        Ok(())
    }
}
impl KernelContext {

    pub async fn spawn_controller_port_event_loop<C: ControllerPort>(
        &self,
        mut controller_port: C,
    ) -> Result<(), ConnectError> {
        let context = self.clone();
        let take_over = start_up_connection(&mut controller_port, &context.kernel_config.controller.connect).await?;
        if let Some(handle) = context.controller_handle.write().await.take() {
            tokio::spawn(async move {
                let result = handle.take_over(take_over).await;
                if let Err(error) = result {
                    tracing::error!("Failed to report been took over to old controller: {}", error);
                } else {
                    tracing::info!("Reported been took over to old controller");
                }
            });
        }
        
        Ok(())
    }
}
