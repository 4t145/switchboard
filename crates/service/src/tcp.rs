use std::{
    net::SocketAddr,
    pin::{self, Pin},
    sync::Arc,
};

use futures::future::BoxFuture;
use rustls::server::Acceptor;
use tokio::{
    io::{self, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::{TcpListener as TokioTcpListener, TcpStream},
};
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

use crate::tcp::tls::MaybeTlsStream;

pub mod listener;
pub mod tls;

pub trait AsyncStream: AsyncRead + AsyncWrite + Unpin + Send + 'static {}
pub struct BoxedAsyncStream(Box<dyn AsyncStream>);

impl BoxedAsyncStream {
    pub fn new<S: AsyncStream>(stream: S) -> Self {
        Self(Box::new(stream))
    }
    pub fn project(self: Pin<&mut Self>) -> Pin<&mut dyn AsyncStream> {
        unsafe { self.map_unchecked_mut(|s| &mut *s.0) }
    }
}

impl AsyncRead for BoxedAsyncStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.project().poll_read(cx, buf)
    }
}

impl AsyncWrite for BoxedAsyncStream {
    fn poll_write(
        self: pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.project().poll_write(cx, buf)
    }

    fn poll_flush(
        self: pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.project().poll_flush(cx)
    }

    fn poll_shutdown(
        self: pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.project().poll_shutdown(cx)
    }

    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.project().poll_write_vectored(cx, bufs)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send + 'static> AsyncStream for S {}

#[derive(Clone)]
pub struct TcpConnectionContext {
    pub peer_addr: SocketAddr,
    pub ct: CancellationToken,
    // optional tls acceptor, service will decide to use or not.
    pub tls_acceptor: Option<TlsAcceptor>,
}

pub trait TcpService: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn serve(self: Arc<Self>, accepted: TcpAccepted) -> BoxFuture<'static, std::io::Result<()>>;
}

pub type BoxedTcpService = Box<dyn TcpService>;
pub type SharedTcpService = Arc<dyn TcpService>;

#[derive(Debug)]
pub struct TcpListener {
    pub inner: TokioTcpListener,
    pub addr: SocketAddr,
    pub tls: Option<Arc<rustls::ServerConfig>>,
}

pub struct TcpAccepted<S = TcpStream> {
    pub stream: S,
    pub context: TcpConnectionContext,
}

impl TcpAccepted {
    pub fn replace_tls(&mut self, config: Arc<rustls::ServerConfig>) {
        self.context.tls_acceptor.replace(config.into());
    }
    pub async fn close_directly(mut self) -> io::Result<()>{
        self.stream.shutdown().await
    }
}

impl TcpListener {
    pub async fn bind(
        addr: SocketAddr,
        tls: Option<Arc<rustls::ServerConfig>>,
    ) -> io::Result<Self> {
        let inner = TokioTcpListener::bind(addr).await?;
        Ok(Self { inner, addr, tls })
    }
    pub async fn accept(&self, ct: &CancellationToken) -> io::Result<TcpAccepted> {
        self.inner
            .accept()
            .await
            .map(|(tcp_stream, peer_addr)| TcpAccepted {
                stream: tcp_stream,
                context: TcpConnectionContext {
                    peer_addr,
                    ct: ct.child_token(),
                    tls_acceptor: self.tls.clone().map(tokio_rustls::TlsAcceptor::from),
                },
            })
    }
}

impl TcpAccepted {}
impl dyn TcpService {}

fn logging_joined_task(
    result: Result<(tokio::task::Id, Result<(), io::Error>), tokio::task::JoinError>,
) {
    match result {
        Ok((task_id, result)) => match result {
            Ok(_) => tracing::debug!(%task_id, "Task finished successfully"),
            Err(error) => tracing::debug!(%task_id, %error, "Task finished with error"),
        },
        Err(error) => {
            tracing::warn!(%error, "Task join error");
        }
    }
}

#[derive(Debug)]
pub struct RunningTcpService {
    bind: SocketAddr,
    ct: CancellationToken,
    join_handle: tokio::task::JoinHandle<()>,
    updater: tokio::sync::watch::Sender<SharedTcpService>,
}

impl std::fmt::Debug for dyn TcpService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();
        f.debug_tuple("DynTcpService").field(&name).finish()
    }
}

impl RunningTcpService {
    pub fn spawn(listener: TcpListener, service: SharedTcpService) -> io::Result<Self> {
        let (service_updater, mut update_receiver) = tokio::sync::watch::channel(service);
        let bind = listener.addr;
        let has_tls = listener.tls.is_some();
        let ct = CancellationToken::new();
        let handle_ct = ct.clone();
        // bind tcp
        let task = async move {
            let mut task_set = tokio::task::JoinSet::new();
            loop {
                let accept_result = tokio::select! {
                    maybe_next = listener.accept(&ct) => {
                        maybe_next
                    }
                    _ = ct.cancelled() => {
                        tracing::debug!("listener cancelled");
                        break;
                    }
                    update = update_receiver.changed() => {
                        match update {
                            Ok(()) => {
                                tracing::info!("TCP service updated");
                            }
                            Err(_) => {
                                tracing::warn!("Service updater channel closed, keeping existing service");
                            }
                        }
                        continue;
                    }
                    finished = task_set.join_next_with_id(), if !task_set.is_empty()=> {
                        if let Some(result) = finished {
                            logging_joined_task(result);
                        };
                        continue;
                    }
                };
                match accept_result {
                    Ok(accepted) => {
                        task_set.spawn(update_receiver.borrow().clone().serve(accepted));
                    }
                    Err(accept_error) => {
                        tracing::warn!(error=%accept_error, "Failed to accept connection");
                        continue;
                    }
                }
            }
            tracing::debug!("listen loop exited");
            while let Some(result) = task_set.join_next_with_id().await {
                logging_joined_task(result);
            }
        };
        let span = tracing::warn_span!("running-tcp-service-loop", has_tls, %bind);
        let join_handle = tokio::spawn(task.instrument(span));
        Ok(Self {
            bind,
            ct: handle_ct,
            join_handle,
            updater: service_updater,
        })
    }
    pub fn update_service(
        &self,
        service: SharedTcpService,
    ) -> Result<(), tokio::sync::watch::error::SendError<SharedTcpService>> {
        self.updater.send(service)
    }
    pub fn bind(&self) -> SocketAddr {
        self.bind
    }
    pub async fn wait(self) -> Result<(), tokio::task::JoinError> {
        self.join_handle.await
    }
    pub async fn cancel(self) -> Result<(), tokio::task::JoinError> {
        self.ct.cancel();
        self.join_handle.await
    }
}
