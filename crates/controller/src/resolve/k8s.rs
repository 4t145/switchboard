use bincode::config;
use k8s_openapi::api::core::v1::Secret;
use kube::Api;
use switchboard_custom_config::K8sResource;
use switchboard_model::TlsCertParams;
mod service_config;
use crate::resolve::{ResolveServiceConfigError, ServiceConfigResolver, k8s::service_config::K8sServiceBuildConfig};
use service_config::K8sGatewayResourceError;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Default)]
pub struct K8sResolveConfig {}

pub struct K8sServiceConfigResolver;

impl K8sServiceConfigResolver {
    pub async fn resolve_service_config_from_k8s(
        &self,
        context: crate::ControllerContext,
        config: K8sServiceBuildConfig,
    ) -> Result<switchboard_model::ServiceConfig, K8sGatewayResourceError> {
        let client = context
            .get_k8s_client()
            .ok_or(K8sGatewayResourceError::NoK8sClient)?;
        let builder = service_config::K8sServiceConfigBuilder::new(client, config);
        let svc_config = builder.build_config_from_k8s().await?;
        Ok(svc_config)
    }
}


impl ServiceConfigResolver for K8sServiceConfigResolver {
    fn resolve(
        &self,
        build_config: switchboard_custom_config::SerdeValue,
        context: crate::ControllerContext,
    ) -> futures::future::BoxFuture<
        '_,
        Result<switchboard_model::ServiceConfig, ResolveServiceConfigError>,
    > {
        Box::pin(async move {
            let build_config = build_config.deserialize_into::<K8sServiceBuildConfig>()?;
            let svc_config = self
                .resolve_service_config_from_k8s(context, build_config)
                .await.map_err(ResolveServiceConfigError::resolve_error)?;
            Ok(svc_config)
        })
    }
}
