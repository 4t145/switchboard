use std::time::Duration;

use hyper::rt::Timer;

use crate::service::timeout::TimeoutService;

use super::Layer;

pub struct Timeout<T> {
    pub timeout: Duration,
    pub timeout_message: bytes::Bytes,
    pub timer: T,
}

impl<S, T: Timer> Layer<S> for Timeout<T> {
    type Service = TimeoutService<S, T>;

    fn layer(self, service: S) -> Self::Service {
        TimeoutService {
            timeout: self.timeout,
            timeout_message: self.timeout_message,
            inner: service,
            timer: self.timer,
        }
    }
}
