use bytes::Bytes;
use http::{Response, StatusCode};
use http_body_util::Full;
use hyper::{
    body::Body,
    rt::{Sleep, Timer},
    service::Service,
};
use std::{pin::Pin, task::Poll, time::Duration};

use crate::{response::IntoResponse, utils::Either};
pub struct TimeoutService<S> {
    pub timeout: Duration,
    pub timeout_message: Bytes,
    pub inner: S,
    pub timer: Box<dyn Timer>,
}

pub struct TimeoutResponse {
    pub message: Bytes,
    pub duration: Duration,
}

impl IntoResponse for TimeoutResponse {
    fn into_response(self) -> Response<impl Body> {
        Response::builder()
            .status(StatusCode::GATEWAY_TIMEOUT)
            .body(Full::new(self.message))
            .unwrap()
    }
}

pin_project_lite::pin_project! {
    pub struct TimeoutFuture<Req, S: Service<Req>> {
        timeout: Duration,
        #[pin]
        fut: S::Future,
        sleep: Pin<Box<dyn Sleep>>,
        timeout_message: Bytes
    }
}

impl<Req, S: Service<Req>> Future for TimeoutFuture<Req, S> {
    type Output = Result<Either<S::Response, TimeoutResponse>, S::Error>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        match this.fut.poll(cx) {
            Poll::Pending => match this.sleep.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => Poll::Ready(Ok(Either::Right(TimeoutResponse {
                    message: this.timeout_message.clone(),
                    duration: *this.timeout,
                }))),
            },
            Poll::Ready(output) => Poll::Ready(output.map(Either::Left)),
        }
    }
}

impl<Req, S: Service<Req>> Service<Req> for TimeoutService<S> {
    type Response = Either<S::Response, TimeoutResponse>;

    type Error = S::Error;

    type Future = TimeoutFuture<Req, S>;

    fn call(&self, req: Req) -> Self::Future {
        let fut = self.inner.call(req);
        let sleep = self.timer.sleep(self.timeout);
        let timeout_message = self.timeout_message.clone();
        TimeoutFuture {
            timeout: self.timeout,
            fut,
            sleep,
            timeout_message,
        }
    }
}
