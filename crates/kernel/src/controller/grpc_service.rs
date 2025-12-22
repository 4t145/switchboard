use std::{future::ready, pin::Pin, time::Duration};

use futures::future::BoxFuture;
use switchboard_custom_config::formats::decode_bytes;
use switchboard_kernel_control::kernel::{
    kernel_service_server::{KernelService, KernelServiceServer},
    *,
};
use switchboard_model::Config;

use crate::KernelContext;

#[derive(Clone)]
pub(crate) struct KernelServiceImpl {
    kernel_context: KernelContext,
}

impl KernelServiceImpl {
    pub fn new(kernel_context: &KernelContext) -> Self {
        Self {
            kernel_context: kernel_context.clone(),
        }
    }
}

pub(crate) struct StatusStream {
    kernel_context: KernelContext,
    interval: tokio::time::Interval,
    poll: StatusStreamPoll,
}

pub enum StatusStreamPoll {
    Interval,
    PollRead(BoxFuture<'static, tokio::sync::OwnedRwLockReadGuard<switchboard_model::kernel::KernelState>>),
}

impl StatusStream {
    pub fn new(kernel_context: &KernelContext) -> Self {
        let interval = tokio::time::interval(
            Duration::from_secs(kernel_context.kernel_config.controller.state_report_interval as u64)
            );
        Self {
            kernel_context: kernel_context.clone(),
            interval,
            poll: StatusStreamPoll::Interval,
        }
    }
}

impl tokio_stream::Stream for StatusStream {
    type Item = Result<KernelState, tonic::Status>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match &mut this.poll {
                StatusStreamPoll::Interval => {
                    match Pin::new(&mut this.interval).poll_tick(cx) {
                        std::task::Poll::Ready(_) => {
                            let poll_read = Box::pin(this
                                                            .kernel_context
                                                            .state
                                                            .clone()
                                                            .read_owned());
                            this.poll =
                                StatusStreamPoll::PollRead(poll_read);
                        }
                        std::task::Poll::Pending => {
                            return std::task::Poll::Pending;
                        }
                    }
                }
                StatusStreamPoll::PollRead(poll_read) => {
                    match Pin::new(poll_read).poll(cx) {
                        std::task::Poll::Ready(guard) => {
                            let state = guard.to_owned();
                            let status: KernelState = state.into();
                            this.poll = StatusStreamPoll::Interval;
                            return std::task::Poll::Ready(Some(Ok(status)));
                        }
                        std::task::Poll::Pending => {
                            return std::task::Poll::Pending;
                        }
                    }
                }
            }
        }
    }
}


impl KernelService for KernelServiceImpl {
    type WatchStatusStream = StatusStream;
    fn get_kernel_info<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<GetKernelInfoRequest>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<tonic::Response<KernelInfo>, tonic::Status>>
                + Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let info: KernelInfo = self.kernel_context.kernel_config.info.clone().into();
        let response = tonic::Response::new(info);
        Box::pin(ready(Ok(response)))
    }
    fn update_config<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<UpdateConfigRequest>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = std::result::Result<
                        tonic::Response<UpdateConfigResponse>,
                        tonic::Status,
                    >,
                > + Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let request = request.into_inner();
        let format = request.format;
        let config_data = request.config;
        let version = request.version;
        // parse config
        let decode_result = decode_bytes(format.as_bytes(), config_data.into());
        let config: Config = match decode_result {
            Err(e) => {
                let status = tonic::Status::invalid_argument(format!(
                    "Failed to decode config data in format {}: {}",
                    format, e
                ));
                return Box::pin(ready(Err(status)));
            }
            Ok(config) => config,
        };
        Box::pin(async move {
            let update_result = self.kernel_context.update_config(config).await;
            match update_result {
                Ok(_) => {
                    let response = UpdateConfigResponse { 
                        result: Some(switchboard_kernel_control::kernel::update_config_response::Result::Success(Empty {})) 
                    };
                    Ok(tonic::Response::new(response))
                }
                Err(e) => {
                    let error_stack = 
                        switchboard_model::error::ErrorStack::from_std(e).into();
                    let response = UpdateConfigResponse { 
                        result: Some(switchboard_kernel_control::kernel::update_config_response::Result::Error(error_stack)) 
                    };
                    Ok(tonic::Response::new(response))
                }
            }
        })
    }

    fn watch_status<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<WatchStatusRequest>,
    ) -> ::core::pin::Pin<
        Box<
            dyn Future<Output = Result<tonic::Response<Self::WatchStatusStream>, tonic::Status>>
                + Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let _request = request.into_inner();
        let stream = StatusStream::new(&self.kernel_context);
        let response = tonic::Response::new(stream);
        Box::pin(ready(Ok(response)))
    }
}


impl KernelContext {
    pub(crate) fn build_grpc_server(&self) -> KernelServiceServer<KernelServiceImpl> {
        let kernel_service = KernelServiceImpl::new(self);
        KernelServiceServer::new(kernel_service)
    }
}