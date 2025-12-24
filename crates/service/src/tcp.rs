use std::{
    net::SocketAddr,
    pin::{self, Pin},
    sync::Arc,
};

use futures::future::BoxFuture;
use tokio::{
    io::{self, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::{TcpListener as TokioTcpListener, TcpStream},
};
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;

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
    pub bind: SocketAddr,
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
    ) -> io::Result<Self> {
        let inner = TokioTcpListener::bind(addr).await?;
        Ok(Self { inner, bind: addr })
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
                    tls_acceptor: None,
                },
            })
    }
}