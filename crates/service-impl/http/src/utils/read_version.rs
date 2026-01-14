use futures::ready;
use pin_project_lite::pin_project;
use std::future::Future;
use std::marker::PhantomPinned;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::task::{Context, Poll};
use switchboard_service::utils::rewind::Rewind;
use tokio::io::{self, AsyncRead, ReadBuf};

use bytes::Bytes;

const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
use crate::HttpVersion;

pub(crate) fn read_version<S>(io: S) -> ReadVersion<S>
where
    S: AsyncRead + Unpin,
{
    ReadVersion {
        io: Some(io),
        buf: [MaybeUninit::uninit(); 24],
        filled: 0,
        version: HttpVersion::Http2,
        _pin: PhantomPinned,
    }
}

pin_project! {
    pub(crate) struct ReadVersion<I> {
        io: Option<I>,
        buf: [MaybeUninit<u8>; 24],
        // the amount of `buf` thats been filled
        filled: usize,
        version: HttpVersion,
        // Make this future `!Unpin` for compatibility with async trait methods.
        #[pin]
        _pin: PhantomPinned,
    }
}

impl<S> Future for ReadVersion<S>
where
    S: AsyncRead + Unpin,
{
    type Output = io::Result<(HttpVersion, Rewind<S>)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut buf = ReadBuf::uninit(&mut *this.buf);
        unsafe {
            buf.assume_init(*this.filled);
        }

        // We start as H2 and switch to H1 as soon as we don't have the preface.
        while buf.filled().len() < H2_PREFACE.len() {
            let len = buf.filled().len();
            ready!(Pin::new(this.io.as_mut().unwrap()).poll_read(cx, &mut buf))?;
            *this.filled = buf.filled().len();

            // We starts as H2 and switch to H1 when we don't get the preface.
            if buf.filled().len() == len
                || buf.filled()[len..] != H2_PREFACE[len..buf.filled().len()]
            {
                *this.version = HttpVersion::Http1;
                break;
            }
        }

        let io = this.io.take().unwrap();
        let buf = buf.filled().to_vec();
        Poll::Ready(Ok((
            *this.version,
            Rewind::new_buffered(io, Bytes::from(buf)),
        )))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn test_read_version() {
        // test h2
        let data = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n some http data";
        let cursor = Cursor::new(data);
        let (version, mut rest) = read_version(cursor).await.unwrap();
        assert!(matches!(version, HttpVersion::Http2));
        let mut buffer = Vec::with_capacity(64);
        rest.read_to_end(&mut buffer).await.unwrap();
        assert_eq!(&buffer, data);

        // test h1
        let data = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let cursor = Cursor::new(data);
        let (version, mut rest) = read_version(cursor).await.unwrap();
        assert!(matches!(version, HttpVersion::Http1));
        let mut buffer = Vec::with_capacity(64);
        rest.read_to_end(&mut buffer).await.unwrap();
        assert_eq!(&buffer, data);
    }
}
