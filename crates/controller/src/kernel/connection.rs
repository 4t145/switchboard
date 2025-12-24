use std::sync::Arc;

use crate::kernel::{KernelAddr, grpc_client::KernelGrpcClient};
use futures::StreamExt;
use switchboard_kernel_control::kernel::{GetKernelInfoRequest, WatchStatusRequest};
use switchboard_model::{
    error::ErrorStack,
    kernel::{KernelInfo, KernelState},
};

#[derive(Clone)]
pub struct KernelGrpcConnection {
    pub addr: Arc<KernelAddr>,
    pub client: KernelGrpcClient,
    pub info_cache: Arc<KernelInfo>,
    pub state_receiver: tokio::sync::watch::Receiver<KernelState>,
    pub state_updater: tokio::sync::watch::Sender<KernelState>,
}

#[derive(Debug, thiserror::Error)]
pub enum KernelGrpcConnectionError {
    #[error("gRPC connection error: {0}")]
    GrpcConnectError(#[from] tonic::transport::Error),
    #[error("gRPC request error: {0}")]
    GrpcRequestError(#[from] tonic::Status),
    #[error("Kernel update config error: {0}")]
    UpdateConfigError(#[from] ErrorStack),
    #[error("Kernel state parse error: {0}")]
    StateParseError(#[from] switchboard_kernel_control::TryFromProtoKernelStateError),
}

impl KernelGrpcConnection {
    pub async fn connect(addr: KernelAddr) -> Result<Self, KernelGrpcConnectionError> {
        let mut client = addr.connect_grpc().await?;
        let addr = Arc::new(addr);
        let info = {
            let response = client.get_kernel_info(GetKernelInfoRequest {}).await?;
            KernelInfo::from(response.into_inner())
        };
        let state = {
            let response = client
                .get_current_state(switchboard_kernel_control::kernel::GetCurrentStateRequest {})
                .await?;
            let proto_state = response.into_inner();
            KernelState::try_from(proto_state)?
        };
        let (state_updater, state_receiver) = tokio::sync::watch::channel(state.clone());
        Ok(Self {
            addr,
            client,
            info_cache: Arc::new(info),
            state_receiver,
            state_updater,
        })
    }
    pub fn get_info_from_cache(&self) -> KernelInfo {
        self.info_cache.as_ref().clone()
    }
    pub fn get_latest_state_from_cache(&self) -> KernelState {
        self.state_receiver.borrow().clone()
    }
    pub async fn get_state_stream(
        &mut self,
    ) -> Result<
        impl futures::Stream<Item = Result<KernelState, KernelGrpcConnectionError>>,
        KernelGrpcConnectionError,
    > {
        let stream = self
            .client
            .watch_status(WatchStatusRequest {})
            .await?
            .into_inner();
        let state_updater = self.state_updater.clone();
        Ok(stream.map(move |result| match result {
            Ok(state) => {
                let state = KernelState::try_from(state)?;
                state_updater.send(state.clone()).ok();
                Ok(state)
            }
            Err(e) => Err(e.into()),
        }))
    }
    pub async fn get_info(&mut self) -> Result<KernelInfo, KernelGrpcConnectionError> {
        let response = self.client.get_kernel_info(GetKernelInfoRequest {}).await?;
        let info = KernelInfo::from(response.into_inner());
        Ok(info)
    }
    pub async fn get_current_state(&mut self) -> Result<KernelState, KernelGrpcConnectionError> {
        let response = self
            .client
            .get_current_state(switchboard_kernel_control::kernel::GetCurrentStateRequest {})
            .await?;
        let proto_state = response.into_inner();
        let state = KernelState::try_from(proto_state)?;
        self.state_updater.send(state.clone()).ok();
        Ok(state)
    }
    pub async fn update_config(
        &mut self,
        new_config: &switchboard_model::Config,
    ) -> Result<(), KernelGrpcConnectionError> {
        let version = new_config.digest_sha256_base64();
        const FORMAT: &str = "bincode";
        let config_bytes = switchboard_custom_config::formats::encode_bytes(FORMAT, new_config)
            .map_err(|e| tonic::Status::internal(format!("Config encode error: {}", e)))?;
        let request = switchboard_kernel_control::kernel::UpdateConfigRequest {
            format: FORMAT.to_string(),
            config: config_bytes.to_vec(),
            version,
        };
        let response = self.client.update_config(request).await?.into_inner();
        match response.result {
            Some(switchboard_kernel_control::kernel::update_config_response::Result::Success(
                _,
            )) => Ok(()),
            Some(switchboard_kernel_control::kernel::update_config_response::Result::Error(
                error_stack,
            )) => {
                let error_stack: ErrorStack = error_stack.into();
                Err(KernelGrpcConnectionError::UpdateConfigError(
                    error_stack.into(),
                ))
            }
            None => Err(KernelGrpcConnectionError::GrpcRequestError(
                tonic::Status::internal("Kernel returned empty result on config update"),
            )),
        }
    }
}
