use futures::ready;
use pin_project_lite::pin_project;
use rustls::server::Acceptor;
use std::future::Future;
use std::io::Cursor;
use std::marker::PhantomPinned;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncRead, ReadBuf};
use tokio_util::bytes::Bytes;

use crate::utils::rewind::Rewind;

const MAX_CLIENT_HELLO_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone)]
pub struct OwnedClientHello {
    pub server_name: Option<String>,
    pub signature_schemes: Vec<rustls::SignatureScheme>,
    pub alpn: Option<Vec<Vec<u8>>>,
    pub server_cert_types: Option<Vec<rustls::server::CertificateType>>,
    pub client_cert_types: Option<Vec<rustls::server::CertificateType>>,
    pub cipher_suites: Vec<rustls::CipherSuite>,
    pub certificate_authorities: Option<Vec<rustls::DistinguishedName>>,
    pub named_groups: Option<Vec<rustls::NamedGroup>>,
}

pub(crate) fn read_client_hello<S>(io: S) -> ReadClientHello<S>
where
    S: AsyncRead + Unpin,
{
    ReadClientHello {
        io: Some(io),
        buf: [MaybeUninit::uninit(); MAX_CLIENT_HELLO_BYTES],
        filled: 0,
        _pin: PhantomPinned,
    }
}

pin_project! {
    pub(crate) struct ReadClientHello<I> {
        io: Option<I>,
        buf: [MaybeUninit<u8>; MAX_CLIENT_HELLO_BYTES],
        filled: usize,
        #[pin]
        _pin: PhantomPinned,
    }
}

enum ParseState {
    NeedMore,
    Done(OwnedClientHello),
    NotTls,
}

fn parse_client_hello(input: &[u8]) -> ParseState {
    let mut acceptor = Acceptor::default();
    let mut cursor = Cursor::new(input);

    if acceptor.read_tls(&mut cursor).is_err() {
        return ParseState::NotTls;
    }

    match acceptor.accept() {
        Ok(Some(accepted)) => ParseState::Done(to_owned_client_hello(accepted.client_hello())),
        Ok(None) => ParseState::NeedMore,
        Err((_err, _alert)) => ParseState::NotTls,
    }
}

fn to_owned_client_hello(client_hello: rustls::server::ClientHello<'_>) -> OwnedClientHello {
    OwnedClientHello {
        server_name: client_hello.server_name().map(std::borrow::ToOwned::to_owned),
        signature_schemes: client_hello.signature_schemes().to_vec(),
        alpn: client_hello
            .alpn()
            .map(|protocols| protocols.map(std::borrow::ToOwned::to_owned).collect()),
        server_cert_types: client_hello.server_cert_types().map(std::borrow::ToOwned::to_owned),
        client_cert_types: client_hello.client_cert_types().map(std::borrow::ToOwned::to_owned),
        cipher_suites: client_hello.cipher_suites().to_vec(),
        certificate_authorities: client_hello
            .certificate_authorities()
            .map(std::borrow::ToOwned::to_owned),
        named_groups: client_hello.named_groups().map(std::borrow::ToOwned::to_owned),
    }
}

impl<S> Future for ReadClientHello<S>
where
    S: AsyncRead + Unpin,
{
    type Output = io::Result<(Option<OwnedClientHello>, Rewind<S>)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut buf = ReadBuf::uninit(&mut *this.buf);
        // SAFETY: first `filled` bytes were initialized by prior reads.
        unsafe {
            buf.assume_init(*this.filled);
        }

        loop {
            let parsed = match parse_client_hello(buf.filled()) {
                ParseState::Done(client_hello) => Some(Some(client_hello)),
                ParseState::NotTls => Some(None),
                ParseState::NeedMore => None,
            };

            if let Some(client_hello) = parsed {
                let buffered = buf.filled().to_vec();
                let io = this
                    .io
                    .take()
                    .ok_or_else(|| io::Error::other("missing stream in ReadClientHello"))?;
                let rewind = Rewind::new_buffered(io, Bytes::from(buffered));
                return Poll::Ready(Ok((client_hello, rewind)));
            }

            if buf.filled().len() >= MAX_CLIENT_HELLO_BYTES {
                let buffered = buf.filled().to_vec();
                let io = this
                    .io
                    .take()
                    .ok_or_else(|| io::Error::other("missing stream in ReadClientHello"))?;
                let rewind = Rewind::new_buffered(io, Bytes::from(buffered));
                return Poll::Ready(Ok((None, rewind)));
            }

            let before = buf.filled().len();
            let io = this
                .io
                .as_mut()
                .ok_or_else(|| io::Error::other("missing stream in ReadClientHello"))?;
            ready!(Pin::new(io).poll_read(cx, &mut buf))?;
            *this.filled = buf.filled().len();

            if buf.filled().len() == before {
                let buffered = buf.filled().to_vec();
                let io = this
                    .io
                    .take()
                    .ok_or_else(|| io::Error::other("missing stream in ReadClientHello"))?;
                let rewind = Rewind::new_buffered(io, Bytes::from(buffered));
                return Poll::Ready(Ok((None, rewind)));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rustls::pki_types::ServerName;
    use std::io::Cursor;
    use std::sync::Arc;
    use tokio::io::AsyncReadExt;

    fn build_client_hello(host: &str, alpn: Option<Vec<Vec<u8>>>) -> Vec<u8> {
        let mut client_config = rustls::ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();

        if let Some(alpn_protocols) = alpn {
            client_config.alpn_protocols = alpn_protocols;
        }

        let server_name = ServerName::try_from(host)
            .map(|name| name.to_owned())
            .expect("server name should be valid");

        let mut conn = rustls::ClientConnection::new(Arc::new(client_config), server_name)
            .expect("client connection should be created");

        let mut out = Vec::new();
        while conn.wants_write() {
            let written = conn
                .write_tls(&mut out)
                .expect("writing client hello bytes should succeed");
            if written == 0 {
                break;
            }
        }

        out
    }

    #[tokio::test]
    async fn test_read_client_hello_with_full_fields() {
        let data = build_client_hello(
            "example.com",
            Some(vec![b"h2".to_vec(), b"http/1.1".to_vec()]),
        );
        let cursor = Cursor::new(data.clone());
        let (client_hello, mut rest) = read_client_hello(cursor)
            .await
            .expect("read_client_hello should succeed");

        let client_hello = client_hello.expect("client hello should be parsed");

        assert_eq!(client_hello.server_name.as_deref(), Some("example.com"));
        assert_eq!(client_hello.alpn, Some(vec![b"h2".to_vec(), b"http/1.1".to_vec()]));
        assert!(!client_hello.signature_schemes.is_empty());
        assert!(!client_hello.cipher_suites.is_empty());

        let mut buffer = Vec::new();
        rest.read_to_end(&mut buffer)
            .await
            .expect("rewind stream should be readable");
        assert_eq!(buffer, data);
    }

    #[tokio::test]
    async fn test_read_client_hello_non_tls() {
        let data = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n".to_vec();
        let cursor = Cursor::new(data.clone());
        let (client_hello, mut rest) = read_client_hello(cursor)
            .await
            .expect("read_client_hello should succeed");

        assert!(client_hello.is_none());

        let mut buffer = Vec::new();
        rest.read_to_end(&mut buffer)
            .await
            .expect("rewind stream should be readable");
        assert_eq!(buffer, data);
    }

    #[tokio::test]
    async fn test_read_client_hello_truncated() {
        let full = build_client_hello("example.com", None);
        let data = full[..8].to_vec();
        let cursor = Cursor::new(data.clone());
        let (client_hello, mut rest) = read_client_hello(cursor)
            .await
            .expect("read_client_hello should succeed");

        assert!(client_hello.is_none());

        let mut buffer = Vec::new();
        rest.read_to_end(&mut buffer)
            .await
            .expect("rewind stream should be readable");
        assert_eq!(buffer, data);
    }
}
