use switchboard_kernel_control::kernel::kernel_service_client::KernelServiceClient;

use crate::{ControllerContext, kernel::KernelAddr};
pub type KernelGrpcClient = KernelServiceClient<tonic::transport::Channel>;
impl KernelAddr {
    pub async fn connect_grpc(
        &self,
    ) -> Result<KernelServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
        tracing::info!("Connecting to kernel at {:?}", self);
        let endpoint = match self {
            KernelAddr::Uds(path) => tonic::transport::Endpoint::from_shared(
                path.as_os_str().as_encoded_bytes().to_vec(),
            )?,
            KernelAddr::Http(url) => tonic::transport::Endpoint::from_shared(url.to_string())?,
        };
        let channel = endpoint.connect().await?;
        let client = <KernelServiceClient<tonic::transport::Channel>>::new(channel);
        Ok(client)
    }
}
impl ControllerContext {}
