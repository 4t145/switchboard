use std::{convert::Infallible, time::Duration};

use hyper::rt::Timer;
use hyper_util::rt::TokioTimer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{dynamic_response, flow::filter::{FilterClass, FilterLike}, response::IntoResponse};

pub struct TimeoutFilter {
    pub timeout: Duration,
    pub timeout_message: bytes::Bytes,
    pub timer: Box<dyn Timer + Send + Sync>,
}

impl FilterLike for TimeoutFilter {
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
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TimeoutConfig {
    pub timeout: Duration,
    pub timeout_message: String,
}
pub struct Timeout;

impl FilterClass for Timeout {
    type Filter = TimeoutFilter;
    type Error = Infallible;
    type Config = TimeoutConfig;

    fn id(&self) -> crate::instance::class::ClassId {
        crate::instance::class::ClassId::std("timeout")
    }

    fn construct(&self, config: Self::Config) -> Result<Self::Filter, Self::Error> {
        let timer = Box::new(TokioTimer::new());
        Ok(TimeoutFilter {
            timeout: config.timeout,
            timeout_message: config.timeout_message.into(),
            timer,
        })
    }
}
