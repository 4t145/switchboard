use std::time::Duration;

use hyper::rt::Timer;

use crate::{dynamic_response, flow::filter::FilterType, response::IntoResponse};

pub struct Timeout {
    pub timeout: Duration,
    pub timeout_message: bytes::Bytes,
    pub timer: Box<dyn Timer + Send + Sync>,
}

impl FilterType for Timeout {
    async fn call<'c>(
        self: std::sync::Arc<Self>,
        req: crate::DynRequest,
        ctx: &'c mut crate::flow::FlowContext,
        next: super::Next,
    ) -> crate::DynResponse {
        let result = crate::utils::TimeoutFuture {
            timeout: self.timeout,
            fut: next.call(req, ctx),
            sleep: self.timer.sleep(self.timeout),
            timeout_message: self.timeout_message.clone(),
        }
        .await;
        match result {
            crate::utils::Either::Left(response) => response,
            crate::utils::Either::Right(timeout_response) => {
                dynamic_response(timeout_response.into_response())
            }
        }
    }
}
