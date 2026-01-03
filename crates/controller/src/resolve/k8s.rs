mod service_config;
use crate::resolve::{
    ResolveServiceConfigError, ServiceConfigResolver, k8s::service_config::K8sServiceBuildConfig,
};
use kube::Client;
use service_config::K8sGatewayResourceError;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Default)]
pub struct K8sResolveConfig {}

pub struct K8sServiceConfigResolver;

impl K8sServiceConfigResolver {
    pub async fn resolve_service_config_from_k8s(
        &self,
        config: K8sServiceBuildConfig,
    ) -> Result<switchboard_model::ServiceConfig, K8sGatewayResourceError> {
        let client = Client::try_default().await?;
        let builder = service_config::K8sServiceConfigBuilder::new(client, config);
        let svc_config = builder.build_config_from_k8s().await?;
        Ok(svc_config)
    }
}

impl ServiceConfigResolver for K8sServiceConfigResolver {
    fn resolve(
        &self,
        build_config: switchboard_custom_config::SerdeValue,
        _context: crate::ControllerContext,
    ) -> futures::future::BoxFuture<
        '_,
        Result<switchboard_model::ServiceConfig, ResolveServiceConfigError>,
    > {
        Box::pin(async move {
            let build_config = build_config.deserialize_into::<K8sServiceBuildConfig>()?;
            let svc_config = self
                .resolve_service_config_from_k8s(build_config)
                .await
                .map_err(ResolveServiceConfigError::resolve_error)?;
            Ok(svc_config)
        })
    }
}
