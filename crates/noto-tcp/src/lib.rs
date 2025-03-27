use std::{net::SocketAddr, sync::Arc};

use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    net::TcpListener,
};
use tokio_util::sync::CancellationToken;

pub mod tls;

pub trait TcpService: Send + Clone + 'static {
    fn serve<S>(
        self,
        stream: S,
        peer: SocketAddr,
        ct: CancellationToken,
    ) -> impl Future<Output = std::io::Result<()>> + Send + 'static
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static;
}

pub trait TcpServiceExt: TcpService {
    fn bind(&self, addr: SocketAddr) -> impl Future<Output = io::Result<RunningTcpService>>;
    fn listen(self, listener: TcpListener, ct: CancellationToken) -> impl Future<Output = ()>;

    fn tls(self, config: impl Into<Arc<rustls::ServerConfig>>) -> tls::TlsService<Self> {
        tls::TlsService {
            config: config.into(),
            service: self,
        }
    }
}

impl<S: TcpService> TcpServiceExt for S {
    async fn bind(&self, addr: SocketAddr) -> io::Result<RunningTcpService> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!(%addr, "Listening on TCP");
        let ct = CancellationToken::new();
        let join_handle = tokio::spawn(self.clone().listen(listener, ct.child_token()));
        Ok(RunningTcpService {
            bind: addr,
            ct,
            join_handle,
        })
    }
    async fn listen(self, listener: TcpListener, ct: CancellationToken) {
        let mut task_set = tokio::task::JoinSet::new();
        loop {
            let (stream, peer) = tokio::select! {
                _ = ct.cancelled() => {
                    tracing::info!("Cancellation token triggered, shutting down server");
                    break;
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
                finished = task_set.join_next_with_id() => {
                    if let Some(result) = finished {
                        logging_joined_task(result);
                    };
                    continue;
                }
            };
            let service = self.clone().serve(stream, peer, ct.child_token());
            task_set.spawn(service);
        }
        while let Some(result) = task_set.join_next_with_id().await {
            logging_joined_task(result);
        }
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

pub struct RunningTcpService {
    bind: SocketAddr,
    ct: CancellationToken,
    join_handle: tokio::task::JoinHandle<()>,
}

impl RunningTcpService {
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
