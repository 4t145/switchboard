use std::{
    net::SocketAddr,
    pin::{self, Pin},
    sync::Arc,
};

use futures::future::BoxFuture;
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    net::TcpListener,
};
use tokio_util::sync::CancellationToken;

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

pub trait TcpService: Send + Sync + 'static {
    fn serve<S>(
        self: Arc<Self>,
        stream: S,
        peer: SocketAddr,
        ct: CancellationToken,
    ) -> impl Future<Output = std::io::Result<()>> + Send + 'static
    where
        S: AsyncStream;
}

impl TcpService for dyn DynTcpService {
    fn serve<S>(
        self: Arc<Self>,
        stream: S,
        peer: SocketAddr,
        ct: CancellationToken,
    ) -> impl Future<Output = std::io::Result<()>> + Send + 'static
    where
        S: AsyncStream,
    {
        self.serve(BoxedAsyncStream::new(stream), peer, ct)
    }
}

pub trait DynTcpService: Send + Sync + 'static {
    fn serve(
        self: Arc<Self>,
        stream: BoxedAsyncStream,
        peer: SocketAddr,
        ct: CancellationToken,
    ) -> BoxFuture<'static, std::io::Result<()>>;
}

impl<S: TcpService + ?Sized> DynTcpService for S {
    fn serve(
        self: Arc<Self>,
        stream: BoxedAsyncStream,
        peer: SocketAddr,
        ct: CancellationToken,
    ) -> BoxFuture<'static, std::io::Result<()>> {
        Box::pin(self.serve(stream, peer, ct))
    }
}

pub trait TcpServiceExt: Sized {
    fn bind(&self, addr: SocketAddr) -> impl Future<Output = io::Result<RunningTcpService>>;
    fn listen(&self, listener: TcpListener, ct: CancellationToken) -> impl Future<Output = ()>;

    fn tls(self, config: impl Into<Arc<rustls::ServerConfig>>) -> tls::TlsService<Self> {
        tls::TlsService {
            config: config.into(),
            service: Arc::new(self),
        }
    }
}

async fn listen_with_updater(mut service_updater: tokio::sync::watch::Receiver<Arc<dyn DynTcpService>>, listener: TcpListener, ct: CancellationToken) {
    let mut task_set = tokio::task::JoinSet::new();
    let mut service = service_updater.borrow().clone();
    loop {
        let (stream, peer) = tokio::select! {

            _ = ct.cancelled() => {
                tracing::debug!("Cancellation token triggered, shutting down server");
                break;
            }
            update = service_updater.changed() => {
                match update {
                    Ok(()) => {
                        service = service_updater.borrow().clone();
                        tracing::info!("TCP service updated");
                    }
                    Err(_) => {
                        tracing::warn!("Service updater channel closed, keeping existing service");
                    }
                }
                continue;
            }
            next_income_result = listener.accept() => {
                match next_income_result {
                    Err(error) => {
                        tracing::error!(%error, "Failed to accept connection");
                        continue;
                    }
                    Ok((stream, peer)) => {
                        tracing::debug!(%peer, "new connection");
                        (stream, peer)
                    }
                }
            }
            finished = task_set.join_next_with_id(), if !task_set.is_empty()=> {
                if let Some(result) = finished {
                    logging_joined_task(result);
                };
                continue;
            }
        };
        let service = service
            .clone()
            .serve(BoxedAsyncStream::new(stream), peer, ct.child_token());
        task_set.spawn(service);
    }
    while let Some(result) = task_set.join_next_with_id().await {
        logging_joined_task(result);
    }
}
impl dyn DynTcpService {
    pub async fn bind(self: Arc<Self>, addr: SocketAddr) -> io::Result<RunningTcpService> {
        let listener = TcpListener::bind(addr).await?;
        tracing::debug!(%addr, "Listening on TCP");
        let ct = CancellationToken::new();
        let service = self.clone();
        let (updater, update_receiver) = tokio::sync::watch::channel(service);
        let join_handle = tokio::spawn({
            let ct = ct.child_token();
            async move { listen_with_updater(update_receiver, listener, ct).await }
        });
        Ok(RunningTcpService {
            bind: addr,
            ct,
            join_handle,
            updater,
        })
    }
}

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
    updater: tokio::sync::watch::Sender<Arc<dyn DynTcpService>>,
}

impl std::fmt::Debug for dyn DynTcpService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynTcpService")
    }
}

impl RunningTcpService {
    pub fn update_service(&self, service: Arc<dyn DynTcpService>) -> Result<(), tokio::sync::watch::error::SendError<Arc<dyn DynTcpService>>> {
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
