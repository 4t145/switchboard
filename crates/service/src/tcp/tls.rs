use crate::tcp::{AsyncStream, TcpAccepted};
use tokio::io::{self, AsyncRead, AsyncWrite};

impl<S> TcpAccepted<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn maybe_tls(self) -> io::Result<TcpAccepted<MaybeTlsStream<S>>> {
        let Self { stream, context } = self;
        match &context.tls_acceptor {
            Some(tls_acceptor) => {
                let tls_stream = tls_acceptor.accept(stream).await?;
                Ok(TcpAccepted {
                    stream: MaybeTlsStream::Tls(Box::new(tls_stream)),
                    context,
                })
            }
            None => Ok(TcpAccepted {
                stream: MaybeTlsStream::Plain(stream),
                context,
            }),
        }
    }
}

pub enum MaybeTlsStream<S> {
    Tls(Box<tokio_rustls::server::TlsStream<S>>),
    Plain(S),
}

impl<S> MaybeTlsStream<S> {
    pub fn new_tls(stream: tokio_rustls::server::TlsStream<S>) -> Self {
        MaybeTlsStream::Tls(Box::new(stream))
    }

    pub fn new_plain(stream: S) -> Self {
        MaybeTlsStream::Plain(stream)
    }

    pub fn is_tls(&self) -> bool {
        matches!(self, MaybeTlsStream::Tls(_))
    }

    pub fn server_name(&self) -> Option<&str> {
        match self {
            MaybeTlsStream::Tls(tls_stream) => {
                let (_, session) = tls_stream.get_ref();
                session.server_name()
            }
            MaybeTlsStream::Plain(_) => None,
        }
    }

    pub fn alpn_protocol(&self) -> Option<&[u8]> {
        match self {
            MaybeTlsStream::Tls(tls_stream) => {
                let (_, session) = tls_stream.get_ref();
                session.alpn_protocol()
            }
            MaybeTlsStream::Plain(_) => None,
        }
    }
}

impl<S> MaybeTlsStream<S>
where
    S: AsyncStream,
{
    // pub fn into_boxed(self) -> MaybeTlsStream<BoxedAsyncStream> {
    //     match self {
    //         MaybeTlsStream::Tls(tls_stream) => {
    //             let (io, c) = tls_stream.into_inner();
    //             let io = BoxedAsyncStream::new(io);
    //             let tls_stream = tokio_rustls::server::TlsStream::from(value);
    //             MaybeTlsStream::Tls(tls_stream)
    //         }
    //         MaybeTlsStream::Plain(plain_stream) => {
    //             MaybeTlsStream::Plain(BoxedAsyncStream::new(plain_stream))
    //         }
    //     }
    // }
}

impl<S> AsyncRead for MaybeTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        match self.get_mut() {
            MaybeTlsStream::Tls(tls_stream) => std::pin::Pin::new(tls_stream).poll_read(cx, buf),
            MaybeTlsStream::Plain(plain_stream) => {
                std::pin::Pin::new(plain_stream).poll_read(cx, buf)
            }
        }
    }
}

impl<S> AsyncWrite for MaybeTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        match self.get_mut() {
            MaybeTlsStream::Tls(tls_stream) => std::pin::Pin::new(tls_stream).poll_write(cx, buf),
            MaybeTlsStream::Plain(plain_stream) => {
                std::pin::Pin::new(plain_stream).poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        match self.get_mut() {
            MaybeTlsStream::Tls(tls_stream) => std::pin::Pin::new(tls_stream).poll_flush(cx),
            MaybeTlsStream::Plain(plain_stream) => std::pin::Pin::new(plain_stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        match self.get_mut() {
            MaybeTlsStream::Tls(tls_stream) => std::pin::Pin::new(tls_stream).poll_shutdown(cx),
            MaybeTlsStream::Plain(plain_stream) => {
                std::pin::Pin::new(plain_stream).poll_shutdown(cx)
            }
        }
    }
}
