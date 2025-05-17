use std::time::Duration;

use hyper::rt::Timer;

use crate::service::timeout::TimeoutService;

use super::Layer;

pub struct Timeout {
    pub timeout: Duration,
    pub timeout_message: bytes::Bytes,
    pub timer: Box<dyn Timer>,
}

impl<S> Layer<S> for Timeout {
    type Service = TimeoutService<S>;

    fn layer(self, service: S) -> Self::Service {
        TimeoutService {
            timeout: self.timeout,
            timeout_message: self.timeout_message,
            inner: service,
            timer: self.timer,
        }
    }
}
