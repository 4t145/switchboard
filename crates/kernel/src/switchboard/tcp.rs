use crate::switchboard::ResourceKey;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use switchboard_service::tcp::{SharedTcpService, TcpAccepted, TcpListener};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

type Tls = Arc<rustls::ServerConfig>;
type EventSender = tokio::sync::mpsc::Sender<TcpSwitchboardEvent>;
type EventReceiver = tokio::sync::mpsc::Receiver<TcpSwitchboardEvent>;
const LOCAL_EVENT_BUFFER_BATCH_SIZE: usize = 1 << 8;

pub(crate) enum TcpSwitchboardEvent {
    NewAccepted {
        from_bind: SocketAddr,
        tcp_accepted: TcpAccepted,
    },
    UpdateRouter(Arc<TcpSwitchboardRouter>),
    Halt,
}

pub(crate) enum TcpSwitchboard {
    Halted(TcpSwitchboardContext),
    Running(TcpSwitchboardHandle),
}
impl TcpSwitchboard {
    pub fn new_halted() -> Self {
        let context = TcpSwitchboardContext::new();
        TcpSwitchboard::Halted(context)
    }
    pub fn ensure_running(&mut self) {
        match self {
            TcpSwitchboard::Halted(context) => {
                let context = std::mem::take(context);
                let handle = context.spawn();
                *self = TcpSwitchboard::Running(handle);
            }
            TcpSwitchboard::Running(_) => { /* already running */ }
        }
    }
    pub fn handle(&self) -> Result<&TcpSwitchboardHandle, crate::Error> {
        match self {
            TcpSwitchboard::Halted(_) => Err(TcpSwitchboardError::TaskHalted.into()),
            TcpSwitchboard::Running(handle) => Ok(handle),
        }
    }
    pub fn handle_mut(&mut self) -> Result<&mut TcpSwitchboardHandle, crate::Error> {
        match self {
            TcpSwitchboard::Halted(_) => Err(TcpSwitchboardError::TaskHalted.into()),
            TcpSwitchboard::Running(handle) => Ok(handle),
        }
    }
}

pub(crate) struct TcpSwitchboardContext {
    pub(crate) event_receiver: EventReceiver,
    pub(crate) event_sender: EventSender,
    pub(crate) router: Arc<TcpSwitchboardRouter>,
    pub(crate) task_set: tokio::task::JoinSet<tokio::io::Result<()>>,
    pub(crate) local_event_buffer: Vec<TcpSwitchboardEvent>,
}

#[derive(Clone)]
pub(crate) struct TcpSwitchboardRouter {
    pub(crate) tlss: HashMap<ResourceKey, Tls>,
    pub(crate) tcp_services: HashMap<ResourceKey, SharedTcpService>,
    pub(crate) routes: HashMap<SocketAddr, TcpRoute>,
    // pub listeners: HashMap<SocketAddr, TcpListenerTask>,
}

impl TcpSwitchboardRouter {
    pub fn new() -> Self {
        Self {
            tlss: HashMap::default(),
            tcp_services: HashMap::default(),
            // listeners: HashMap::new(),
            routes: HashMap::new(),
        }
    }
    fn get_service(&self, bind: &SocketAddr) -> Option<(SharedTcpService, Option<Tls>)> {
        let route = self.routes.get(bind)?;
        let service = self.tcp_services.get(&route.service)?;
        let tls = route.tls.as_ref().and_then(|k| self.tlss.get(k));
        Some((service.clone(), tls.cloned()))
    }
}

pub struct TcpSwitchboardHandle {
    event_sender: EventSender,
    current_router: RwLock<Arc<TcpSwitchboardRouter>>,
    task_handle: tokio::task::JoinHandle<TcpSwitchboardContext>,
    pub(crate) tcp_listeners: HashMap<SocketAddr, TcpListenerTask>,
}
static UNEXPECTED_HALT_HINT: &str = "tcp switchboard task halted unexpectedly";
#[derive(Debug, thiserror::Error)]
pub enum TcpSwitchboardError {
    #[error("TCP switchboard task is not running")]
    TaskHalted,
}
impl TcpSwitchboardHandle {
    pub async fn create_listener_task(
        &mut self,
        listener: TcpListener,
    ) -> Result<(), crate::Error> {
        let listener_task = TcpListenerTask::spawn(listener, self.event_sender.clone());
        self.tcp_listeners.insert(listener_task.bind, listener_task);
        Ok(())
    }
    pub async fn remove_listener_task(&mut self, bind: &SocketAddr) {
        if let Some(listener_task) = self.tcp_listeners.remove(bind) {
            listener_task.cancel().await;
        }
    }
    pub(crate) async fn halt(self) -> TcpSwitchboardContext {
        tracing::debug!("send halt to tcp switchboard task");
        let send_result = self.event_sender.send(TcpSwitchboardEvent::Halt).await;
        if send_result.is_err() {
            unreachable!("tcp switchboard event channel closed before halt");
        } else {
            let mut context = self
                .task_handle
                .await
                .expect("tcp switchboard task shouldn't panic by design");
            // close all listener tasks
            let mut cancel_listener_join_set = tokio::task::JoinSet::new();
            for (_bind, listener_task) in self.tcp_listeners {
                cancel_listener_join_set.spawn(listener_task.cancel());
            }
            cancel_listener_join_set.join_all().await;
            // wait all serve tasks
            let old_task_set = std::mem::take(&mut context.task_set);
            old_task_set.join_all().await;
            context
        }
    }
    pub(crate) async fn get_current_router(&self) -> Arc<TcpSwitchboardRouter> {
        self.current_router.read().await.clone()
    }
    pub(crate) async fn update_router(
        &self,
        router: Arc<TcpSwitchboardRouter>,
    ) -> Result<(), crate::Error> {
        self.current_router.write().await.clone_from(&router);
        let send_result = self
            .event_sender
            .send(TcpSwitchboardEvent::UpdateRouter(router))
            .await;
        if send_result.is_err() {
            Err(TcpSwitchboardError::TaskHalted.into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TcpRoute {
    pub tls: Option<ResourceKey>,
    pub service: ResourceKey,
}

impl From<switchboard_model::tcp_route::TcpRoute> for TcpRoute {
    fn from(route: switchboard_model::tcp_route::TcpRoute) -> Self {
        TcpRoute {
            tls: route.tls.map(|s| s.into()),
            service: route.service.into(),
        }
    }
}

impl From<&switchboard_model::tcp_route::TcpRoute> for TcpRoute {
    fn from(route: &switchboard_model::tcp_route::TcpRoute) -> Self {
        TcpRoute {
            tls: route.tls.as_deref().map(|s| s.into()),
            service: route.service.as_str().into(),
        }
    }
}

#[derive(Debug)]
pub enum TcpSwitchboardContextQuitReason {
    Halt,
    EventChannelClosed,
}

impl Default for TcpSwitchboardContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpSwitchboardContext {
    pub fn new() -> Self {
        let (event_sender, event_receiver) =
            tokio::sync::mpsc::channel(LOCAL_EVENT_BUFFER_BATCH_SIZE * 4);
        TcpSwitchboardContext {
            event_receiver,
            event_sender,
            task_set: tokio::task::JoinSet::new(),
            router: TcpSwitchboardRouter::new().into(),
            local_event_buffer: Vec::with_capacity(LOCAL_EVENT_BUFFER_BATCH_SIZE),
        }
    }
    fn get_service(&self, bind: &SocketAddr) -> Option<(SharedTcpService, Option<Tls>)> {
        self.router.get_service(bind)
    }
    pub fn spawn(self) -> TcpSwitchboardHandle {
        let event_sender = self.event_sender.clone();
        let current_router = RwLock::new(self.router.clone());
        let span = tracing::warn_span!("tcp-switchboard-event-loop");

        let handle = tokio::spawn(self.run_event_loop().instrument(span));
        TcpSwitchboardHandle {
            event_sender,
            task_handle: handle,
            current_router,
            tcp_listeners: HashMap::new(),
        }
    }
    async fn run_event_loop(mut self) -> Self {
        let mut runtime_local_event_buffer = Vec::new();
        std::mem::swap(
            &mut self.local_event_buffer,
            &mut runtime_local_event_buffer,
        );
        let maybe_failed_loop = async {
            loop {
                for event in runtime_local_event_buffer.drain(..) {
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
                            let peer = tcp_accepted.context.peer_addr;
                            let id = self.task_set.spawn(service.serve(tcp_accepted)).id();
                            tracing::debug!(name: "serve", bind= %from_bind, task_id = %id, peer = %peer);
                        }
                        TcpSwitchboardEvent::UpdateRouter(router) => {
                            self.router = router;
                        }
                        TcpSwitchboardEvent::Halt => return TcpSwitchboardContextQuitReason::Halt,
                    }
                }
                tokio::select! {
                    recv_count = self
                        .event_receiver
                        .recv_many(
                            &mut runtime_local_event_buffer,
                            LOCAL_EVENT_BUFFER_BATCH_SIZE,
                        ) => {
                            if recv_count == 0 {
                                return TcpSwitchboardContextQuitReason::EventChannelClosed;
                            }
                        }
                    finished_task = self.task_set.join_next_with_id(), if !self.task_set.is_empty() => {
                        if let Some(Ok((id, result))) = finished_task {
                            match result {
                                Ok(()) => {
                                    tracing::debug!(name: "serve", task_id = %id, "TCP service task completed successfully");
                                }
                                Err(error) => {
                                    tracing::warn!(name: "serve", task_id = %id, %error, "TCP service task failed");
                                }
                            }
                        };
                    }
                }
            }
        };
        let quit_reason = maybe_failed_loop.await;
        // swap back
        std::mem::swap(
            &mut self.local_event_buffer,
            &mut runtime_local_event_buffer,
        );
        tracing::debug!("tcp switchboard event loop exited: {:?}", quit_reason);
        self
    }
}

pub struct TcpListenerTask {
    pub bind: SocketAddr,
    pub task_handle: tokio::task::JoinHandle<TcpListenerServiceQuitReason>,
    pub ct: CancellationToken,
}
#[derive(Debug)]
pub enum TcpListenerServiceQuitReason {
    Cancelled,
    EventChannelClosed,
}

impl TcpListenerTask {
    pub(crate) async fn cancel(self) -> TcpListenerServiceQuitReason {
        self.ct.cancel();
        self.task_handle
            .await
            .expect("TcpListenerService task shouldn't panic by design")
    }
    pub(crate) fn spawn(tcp_listener: TcpListener, event_sender: EventSender) -> Self {
        let bind = tcp_listener.bind;
        let span = tracing::warn_span!(
            parent: None,
            "tcp-listener",
            bind = %bind,
        );
        let ct = CancellationToken::new();
        let handle_ct = ct.clone();
        let listener_task = async move {
            let quit_reason = loop {
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
                tracing::debug!(name:"tcp-accept", bind = %bind, peer = %accepted.context.peer_addr, "Accepted new TCP connection");
                if event_sender
                    .send(TcpSwitchboardEvent::NewAccepted {
                        from_bind: bind,
                        tcp_accepted: accepted,
                    })
                    .await
                    .is_err()
                {
                    break TcpListenerServiceQuitReason::EventChannelClosed;
                }
            };
            tracing::debug!("tcp listener task exited: {:?}", quit_reason);
            quit_reason
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

impl crate::KernelContext {
    pub async fn shutdown_tcp_switchboard(&self) {
        let old_switch_board = {
            let mut wg = self.tcp_switchboard.write().await;
            std::mem::replace(&mut *wg, TcpSwitchboard::new_halted())
        };
        match old_switch_board {
            TcpSwitchboard::Halted(_) => { /* already halted */ }
            TcpSwitchboard::Running(handle) => {
                handle.halt().await;
            }
        }
    }
}
