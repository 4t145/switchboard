use std::str::FromStr;

use k8s_openapi::api::core::v1::Secret;
use kube::Api;
use switchboard_custom_config::K8sResource;
use switchboard_model::TlsCertParams;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Default)]
pub struct K8sResolveConfig {

}



pub struct K8sResolver {
    context: crate::ControllerContext,
}

impl K8sResolver {
    pub fn new(context: crate::ControllerContext) -> Self {
        Self { context }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum K8sResolveError {
    #[error("no k8s client")]
    NoK8sClient,
    #[error("k8s error {0}")]
    K8sError(#[from] kube::Error),
    #[error("tls cert params error {0}")]
    TlsCertParamsError(#[from] switchboard_model::tls::TlsCertParamsError),
}
impl K8sResolver {
    pub async fn fetch_tls_cert_params(
        &self,
        resource: &K8sResource,
    ) -> Result<TlsCertParams, K8sResolveError> {
        let client = match self.context.get_k8s_client() {
            Some(c) => c,
            None => return Err(K8sResolveError::NoK8sClient),
        };
        let secrets: Api<Secret> = if let Some(ns) = &resource.namespace {
            Api::namespaced(client, ns)
        } else {
            Api::default_namespaced(client)
        };
        let secret = secrets.get(&resource.name).await?;
        let data = secret.data.unwrap_or_default();
        let cert_bytes = data.get("tls.crt").cloned().unwrap_or_default().0;
        let key_bytes = data.get("tls.key").cloned().unwrap_or_default().0;
        let tls_cert_params =
            switchboard_model::tls::TlsCertParams::from_bytes(&cert_bytes, &key_bytes)?;

        Ok(tls_cert_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_fetch_tls_cert_params_no_client() {
        let resource = K8sResource {
            name: "gateway-tls".to_string(),
            namespace: Some("default".to_string()),
        };
        let mut context = crate::ControllerContext::new(Default::default());
        context.try_init_k8s_client().await.unwrap();
        let tls_params = K8sResolver::new(context)
            .fetch_tls_cert_params(&resource)
            .await
            .unwrap();
        println!("tls params key: {:?}", tls_params.key);
        println!("tls params cert: {:?}", tls_params.certs);
    }
}
