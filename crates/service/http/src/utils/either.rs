use std::pin::{Pin, pin};

use bytes::Buf;
use hyper::body::Body;

use crate::response::IntoResponse;

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn project(self: Pin<&mut Self>) -> Either<Pin<&mut L>, Pin<&mut R>> {
        unsafe {
            match self.get_unchecked_mut() {
                Either::Left(l) => Either::Left(Pin::new_unchecked(l)),
                Either::Right(r) => Either::Right(Pin::new_unchecked(r)),
            }
        }
    }
}

impl<L, R> Buf for Either<L, R>
where
    L: Buf,
    R: Buf,
{
    fn remaining(&self) -> usize {
        match self {
            Either::Left(l) => l.remaining(),
            Either::Right(r) => r.remaining(),
        }
    }

    fn chunk(&self) -> &[u8] {
        match self {
            Either::Left(l) => l.chunk(),
            Either::Right(r) => r.chunk(),
        }
    }

    fn advance(&mut self, cnt: usize) {
        match self {
            Either::Left(l) => l.advance(cnt),
            Either::Right(r) => r.advance(cnt),
        }
    }
}

impl<L, R> Body for Either<L, R>
where
    L: Body,
    R: Body,
{
    type Data = Either<L::Data, R::Data>;

    type Error = Either<L::Error, R::Error>;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        match this {
            Either::Left(l) => l
                .poll_frame(cx)
                .map_err(Either::Left)
                .map_ok(|f| f.map_data(Either::Left)),
            Either::Right(r) => r
                .poll_frame(cx)
                .map_err(Either::Right)
                .map_ok(|f| f.map_data(Either::Right)),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Either::Left(l) => l.is_end_stream(),
            Either::Right(r) => r.is_end_stream(),
        }
    }

    fn size_hint(&self) -> hyper::body::SizeHint {
        match self {
            Either::Left(l) => l.size_hint(),
            Either::Right(r) => r.size_hint(),
        }
    }
}

impl<L, R> IntoResponse for Either<L, R>
where
    L: IntoResponse,
    R: IntoResponse,
{
    fn into_response(self) -> http::Response<impl hyper::body::Body> {
        match self {
            Either::Left(l) => l.into_response().map(Either::Left),
            Either::Right(r) => r.into_response().map(Either::Right),
        }
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> http::Response<impl hyper::body::Body> {
        match self {
            Ok(t) => t.into_response().map(Either::Left),
            Err(e) => e.into_response().map(Either::Right),
        }
    }
}
