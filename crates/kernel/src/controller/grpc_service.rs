use std::{future::ready, pin::Pin, time::Duration};

use switchboard_kernel_control::kernel::{
    kernel_service_server::{KernelService, KernelServiceServer},
    *,
};
use switchboard_model::ServiceConfig;

use crate::KernelContext;

const CONFIG_FORMAT_BINCODE: &str = "bincode";

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
}

impl StatusStream {
    pub fn new(kernel_context: &KernelContext) -> Self {
        let interval = tokio::time::interval(Duration::from_secs(
            kernel_context
                .kernel_config
                .controller
                .state_report_interval as u64,
        ));
        Self {
            kernel_context: kernel_context.clone(),
            interval,
        }
    }
}

fn response_success() -> UpdateConfigResponse {
    UpdateConfigResponse {
        result: Some(update_config_response::Result::Success(Empty {})),
    }
}

fn response_error_from_stack(
    error_stack: switchboard_model::error::ErrorStack,
) -> UpdateConfigResponse {
    UpdateConfigResponse {
        result: Some(update_config_response::Result::Error(error_stack.into())),
    }
}

fn decode_config_or_status(
    format: &str,
    config_data: &[u8],
) -> Result<ServiceConfig, tonic::Status> {
    if format != CONFIG_FORMAT_BINCODE {
        return Err(tonic::Status::invalid_argument(format!(
            "only bincode format is supported currently, but got format {}",
            format
        )));
    }
    bincode::decode_from_slice(config_data, bincode::config::standard())
        .map(|(config, _)| config)
        .map_err(|e| {
            tonic::Status::invalid_argument(format!(
                "Failed to decode config data in format {}: {}",
                format, e
            ))
        })
}

fn validate_version_or_status(
    controller_send_version: &str,
    local_calculated_version: &str,
) -> Result<(), tonic::Status> {
    if controller_send_version != local_calculated_version {
        return Err(tonic::Status::invalid_argument(format!(
            "Config version mismatch: controller sent version {}, but calculated version is {}",
            controller_send_version, local_calculated_version
        )));
    }
    Ok(())
}

impl tokio_stream::Stream for StatusStream {
    type Item = Result<KernelState, tonic::Status>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this
            .kernel_context
            .state_receiver
            .has_changed()
            .unwrap_or_else(|err| {
                tracing::warn!("State channel status check failed: {}", err);
                false
            })
        {
            let state = this.kernel_context.get_state();
            let status: KernelState = state.into();
            return std::task::Poll::Ready(Some(Ok(status)));
        } else {
            // poll interval
            match Pin::new(&mut this.interval).poll_tick(cx) {
                std::task::Poll::Ready(_) => {
                    let state = this.kernel_context.get_state();
                    let status: KernelState = state.into();
                    return std::task::Poll::Ready(Some(Ok(status)));
                }
                std::task::Poll::Pending => {
                    return std::task::Poll::Pending;
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
        let config = match decode_config_or_status(&format, &request.config) {
            Ok(config) => config,
            Err(status) => return Box::pin(ready(Err(status))),
        };
        let local_calculated_version = config.digest_sha256_base64();
        if let Err(status) = validate_version_or_status(&request.version, &local_calculated_version)
        {
            return Box::pin(ready(Err(status)));
        }
        Box::pin(async move {
            let update_result = self.kernel_context.update_config(config).await;
            match update_result {
                Ok(_) => Ok(tonic::Response::new(response_success())),
                Err(e) => {
                    let error_stack = switchboard_model::error::ErrorStack::from_std(e);
                    Ok(tonic::Response::new(response_error_from_stack(error_stack)))
                }
            }
        })
    }

    fn prepare_config<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<PrepareConfigRequest>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = std::result::Result<
                        tonic::Response<PrepareConfigResponse>,
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
        let config = match decode_config_or_status(&format, &request.config) {
            Ok(config) => config,
            Err(status) => return Box::pin(ready(Err(status))),
        };
        let local_calculated_version = config.digest_sha256_base64();
        if let Err(status) = validate_version_or_status(&request.version, &local_calculated_version)
        {
            return Box::pin(ready(Err(status)));
        }
        let txn_id = request.txn_id;
        let version = request.version;
        let ttl_secs = if request.ttl_secs == 0 {
            None
        } else {
            Some(request.ttl_secs)
        };
        Box::pin(async move {
            match self
                .kernel_context
                .prepare_config(txn_id, config, version, ttl_secs)
                .await
            {
                Ok(_) => Ok(tonic::Response::new(PrepareConfigResponse {
                    result: Some(prepare_config_response::Result::Success(Empty {})),
                })),
                Err(e) => {
                    let error_stack = switchboard_model::error::ErrorStack::from_std(e).into();
                    Ok(tonic::Response::new(PrepareConfigResponse {
                        result: Some(prepare_config_response::Result::Error(error_stack)),
                    }))
                }
            }
        })
    }

    fn commit_config<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<CommitConfigRequest>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = std::result::Result<
                        tonic::Response<CommitConfigResponse>,
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
        Box::pin(async move {
            match self
                .kernel_context
                .commit_config(&request.txn_id, &request.version)
                .await
            {
                Ok(_) => Ok(tonic::Response::new(CommitConfigResponse {
                    result: Some(commit_config_response::Result::Success(Empty {})),
                })),
                Err(e) => {
                    let error_stack = switchboard_model::error::ErrorStack::from_std(e).into();
                    Ok(tonic::Response::new(CommitConfigResponse {
                        result: Some(commit_config_response::Result::Error(error_stack)),
                    }))
                }
            }
        })
    }

    fn abort_config<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<AbortConfigRequest>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = std::result::Result<
                        tonic::Response<AbortConfigResponse>,
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
        Box::pin(async move {
            match self.kernel_context.abort_config(&request.txn_id).await {
                Ok(_) => Ok(tonic::Response::new(AbortConfigResponse {
                    result: Some(abort_config_response::Result::Success(Empty {})),
                })),
                Err(e) => {
                    let error_stack = switchboard_model::error::ErrorStack::from_std(e).into();
                    Ok(tonic::Response::new(AbortConfigResponse {
                        result: Some(abort_config_response::Result::Error(error_stack)),
                    }))
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

    fn get_current_state<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<switchboard_kernel_control::kernel::GetCurrentStateRequest>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = std::result::Result<
                        tonic::Response<switchboard_kernel_control::kernel::KernelState>,
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
        let current_state = self.kernel_context.get_state();
        let proto_state: switchboard_kernel_control::kernel::KernelState = current_state.into();
        let response = tonic::Response::new(proto_state);
        Box::pin(ready(Ok(response)))
    }
}

impl KernelContext {
    pub(crate) fn build_grpc_server(&self) -> KernelServiceServer<KernelServiceImpl> {
        let kernel_service = KernelServiceImpl::new(self);
        KernelServiceServer::new(kernel_service)
    }
    // pub(crate) fn build_health_grpc_service(&self) -> tonic_health::server::HealthService {
    //     let health_reporter = tonic_health::server::HealthReporter::new();
    // }
}
