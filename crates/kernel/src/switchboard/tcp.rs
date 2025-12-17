use crate::switchboard::ResourceKey;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use switchboard_service::tcp::{
    self, RunningTcpService, SharedTcpService, TcpAccepted, TcpListener,
};
use tokio::net::TcpListener as TokioTcpListener;
use tokio::sync::watch::Sender;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

type Tls = Arc<rustls::ServerConfig>;
type EventSender = tokio::sync::mpsc::UnboundedSender<TcpSwitchboardEvent>;
type EventReceiver = tokio::sync::mpsc::UnboundedReceiver<TcpSwitchboardEvent>;

pub enum ResourceOperation<R> {
    Create { key: ResourceKey, resource: R },
    Update { key: ResourceKey, new_resource: R },
    Remove { key: ResourceKey },
}

pub enum TcpSwitchboardEvent {
    NewAccepted {
        from_bind: SocketAddr,
        tcp_accepted: TcpAccepted,
    },
    TlsOperation(ResourceOperation<Tls>),
    ServiceOperation(ResourceOperation<SharedTcpService>),
    CreateListener(TcpListener),
    RemoveListener(SocketAddr),
    CreateBind {
        bind: SocketAddr,
        service: ResourceKey,
    },
    RemoveBind {
        bind: SocketAddr,
    },
    Halt,
}

pub struct BindItem {
    pub listener: SocketAddr,
}

pub struct TcpSwitchboardContext {
    pub event_receiver: EventReceiver,
    pub tlss: Resources<Tls>,
    pub tcp_services: Resources<SharedTcpService>,
    pub listeners: HashMap<SocketAddr, TcpListenerTask>,
    pub task_set: tokio::task::JoinSet<tokio::io::Result<()>>,
    pub router: HashMap<SocketAddr, TcpRoute>,
}

#[derive(Debug, Clone)]
pub struct TcpRoute {
    pub tls: Option<ResourceKey>,
    pub service: ResourceKey,
}

#[derive(Debug, Clone)]
pub struct Resources<T>(pub HashMap<ResourceKey, T>);

impl<T> Default for Resources<T> {
    fn default() -> Self {
        Resources(Default::default())
    }
}

impl<T> Resources<T> {
    pub fn apply(&mut self, op: ResourceOperation<T>) {
        match op {
            ResourceOperation::Create { key, resource } => {
                self.0.insert(key, resource);
            }
            ResourceOperation::Update { key, new_resource } => {
                self.0.insert(key, new_resource);
            }
            ResourceOperation::Remove { key } => {
                self.0.remove(&key);
            }
        }
    }
}

pub enum TcpSwitchboardContextQuitReason {
    Halt,
    EventChannelClosed,
}
impl TcpSwitchboardContext {
    fn get_service(&self, bind: &SocketAddr) -> Option<(SharedTcpService, Option<Tls>)> {
        let route = self.router.get(bind)?;
        let service = self.tcp_services.0.get(&route.service)?;

        let tls = route.tls.as_ref().and_then(|k| self.tlss.0.get(k));
        Some((service.clone(), tls.cloned()))
    }
    async fn run_event_loop(mut self) -> TcpSwitchboardContextQuitReason {
        let maybe_failed_loop = async move {
            const LOCAL_EVENT_BUFFER_BATCH_SIZE: usize = 1 << 8;
            let mut local_event_buffer = Vec::with_capacity(LOCAL_EVENT_BUFFER_BATCH_SIZE);
            loop {
                if self
                    .event_receiver
                    .recv_many(&mut local_event_buffer, LOCAL_EVENT_BUFFER_BATCH_SIZE)
                    .await
                    == 0
                {
                    return TcpSwitchboardContextQuitReason::EventChannelClosed;
                };
                for event in local_event_buffer.drain(..) {
                    match event {
                        TcpSwitchboardEvent::NewAccepted {
                            from_bind,
                            mut tcp_accepted,
                        } => {
                            let Some((service, tls)) = self.get_service(&from_bind) else {
                                self.task_set.spawn(tcp_accepted.close_directly());
                                continue;
                            };
                            if let Some(tls) = tls {
                                tcp_accepted.replace_tls(tls);
                            };
                            tracing::debug!(name: "serve", bind= %from_bind, peer= %tcp_accepted.context.peer_addr);
                            self.task_set.spawn(service.serve(tcp_accepted));
                        }
                        TcpSwitchboardEvent::TlsOperation(resource_operation) => {
                            self.tlss.apply(resource_operation);
                        }
                        TcpSwitchboardEvent::ServiceOperation(resource_operation) => {
                            self.tcp_services.apply(resource_operation);
                        }
                        TcpSwitchboardEvent::CreateListener(tcp_listener) => todo!(),
                        TcpSwitchboardEvent::RemoveListener(socket_addr) => todo!(),
                        TcpSwitchboardEvent::CreateBind { bind, service } => todo!(),
                        TcpSwitchboardEvent::RemoveBind { bind } => todo!(),
                        TcpSwitchboardEvent::Halt => return TcpSwitchboardContextQuitReason::Halt,
                    }
                }
            }
        };
        todo!()
    }
}

pub struct TcpListenerTask {
    pub bind: SocketAddr,
    pub task_handle: tokio::task::JoinHandle<TcpListenerServiceQuitReason>,
    pub ct: CancellationToken,
}

pub enum TcpListenerServiceQuitReason {
    Cancelled,
    EventChannelClosed,
}

impl TcpListenerTask {
    pub async fn cancel(self) -> TcpListenerServiceQuitReason {
        self.ct.cancel();
        self.task_handle
            .await
            .expect("TcpListenerService task shouldn't panic by design")
    }
    pub fn spawn(tcp_listener: TcpListener, event_sender: EventSender) -> Self {
        let bind = tcp_listener.addr;
        let span = tracing::warn_span!(
            "tcp-listener",
            bind = %bind,
        );
        let ct = CancellationToken::new();
        let handle_ct = ct.clone();
        let listener_task = async move {
            loop {
                let accepted = tokio::select! {
                    accept_result = tcp_listener.accept(&ct) => {
                        match accept_result {
                            Ok(accepted) => accepted,
                            Err(accept_error) => {
                                tracing::warn!(error=%accept_error, "Failed to accept connection");
                                continue;
                            }
                        }
                    },
                    _ = ct.cancelled() => {
                        tracing::debug!("listener cancelled");
                        break TcpListenerServiceQuitReason::Cancelled;
                    }
                };
                if event_sender
                    .send(TcpSwitchboardEvent::NewAccepted {
                        from_bind: bind,
                        tcp_accepted: accepted,
                    })
                    .is_err()
                {
                    break TcpListenerServiceQuitReason::EventChannelClosed;
                }
            }
        }
        .instrument(span);
        let task_handle = tokio::spawn(listener_task);
        TcpListenerTask {
            bind,
            task_handle,
            ct: handle_ct,
        }
    }
}
