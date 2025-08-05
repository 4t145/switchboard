use bytes::Bytes;
use http::{Response, StatusCode};
use http_body_util::Full;
use hyper::{body::Body, rt::Sleep};
use std::{convert::Infallible, pin::Pin, task::Poll, time::Duration};

use crate::{response::IntoResponse, utils::Either};

pub struct TimeoutResponse {
    pub message: Bytes,
    pub duration: Duration,
}

impl IntoResponse for TimeoutResponse {
    type Error = Infallible;
    fn into_response(self) -> Response<impl Body<Data = Bytes, Error = Infallible>> {
        Response::builder()
            .status(StatusCode::GATEWAY_TIMEOUT)
            .body(Full::new(self.message))
            .unwrap()
    }
}

pin_project_lite::pin_project! {
    pub struct TimeoutFuture<F> {
        pub timeout: Duration,
        #[pin]
        pub fut: F,
        pub sleep: Pin<Box<dyn Sleep>>,
        pub timeout_message: Bytes
    }
}

impl<F> Future for TimeoutFuture<F>
where
    F: Future,
{
    type Output = Either<F::Output, TimeoutResponse>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        match this.fut.poll(cx) {
            Poll::Pending => match this.sleep.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => Poll::Ready(Either::Right(TimeoutResponse {
                    message: this.timeout_message.clone(),
                    duration: *this.timeout,
                })),
            },
            Poll::Ready(output) => Poll::Ready(Either::Left(output)),
        }
    }
}
