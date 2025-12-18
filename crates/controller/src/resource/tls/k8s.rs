use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Secret;
use kube::{Api, api::ListParams};
use serde::{Deserialize, Serialize};
use switchboard_model::TlsCertParams;

use crate::DEFAULT_NAMESPACE;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct K8sTlsResourceConfig {
    pub discovery_from_namespaces: Vec<String>,
}

impl Default for K8sTlsResourceConfig {
    fn default() -> Self {
        Self {
            discovery_from_namespaces: vec![DEFAULT_NAMESPACE.to_string()],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum K8sTlsCertResourceError {
    #[error("No Kubernetes client available")]
    NoK8sClient,
    #[error("Kubernetes API error: {0}")]
    K8sApiError(#[from] kube::Error),
    #[error("Missing secret data")]
    MissingSecretData,
    #[error("Missing key in data")]
    MissingKey,
    #[error("Missing cert in data")]
    MissingCert,
}

impl crate::ControllerContext {
    pub async fn discovery_tls_cert_from_k8s(
        &self,
    ) -> Result<BTreeMap<String, K8sTlsCertResource>, K8sTlsCertResourceError> {
        let mut results = BTreeMap::new();
        let Some(config) = &self.controller_config.resource.tls.k8s else {
            return Ok(results);
        };
        let client = match self.get_k8s_client() {
            Some(c) => c,
            None => return Err(K8sTlsCertResourceError::NoK8sClient),
        };
        for namespace in config.discovery_from_namespaces.iter() {
            let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
            let secret_list = secrets
                .list_metadata(&ListParams {
                    field_selector: Some("type=kubernetes.io/tls".to_string()),
                    ..Default::default()
                })
                .await?;
            for secret in secret_list.items {
                let name = secret.metadata.name.unwrap_or_default();
                results.insert(
                    format!("k8s://{}/{}", namespace, name),
                    K8sTlsCertResource {
                        secret_name: name,
                        namespace: namespace.clone(),
                    },
                );
            }
        }
        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct K8sTlsCertResource {
    pub secret_name: String,
    pub namespace: String,
}

impl Default for K8sTlsCertResource {
    fn default() -> Self {
        Self {
            secret_name: "tls".to_string(),
            namespace: "default".to_string(),
        }
    }
}

impl K8sTlsCertResource {
    pub async fn fetch(
        &self,
        context: &crate::ControllerContext,
    ) -> Result<TlsCertParams, K8sTlsCertResourceError> {
        let client = match context.get_k8s_client() {
            Some(c) => c,
            None => return Err(K8sTlsCertResourceError::NoK8sClient),
        };
        let secrets: Api<Secret> = Api::namespaced(client, &self.namespace);
        let secret = secrets.get(&self.secret_name).await?;
        let data = secret
            .data
            .ok_or_else(|| K8sTlsCertResourceError::MissingSecretData)?;
        let cert_bytes = data
            .get("tls.crt")
            .ok_or_else(|| K8sTlsCertResourceError::MissingCert)?
            .0
            .clone();
        let key_bytes = data
            .get("tls.key")
            .ok_or_else(|| K8sTlsCertResourceError::MissingKey)?
            .0
            .clone();
        Ok(TlsCertParams {
            certs: vec![switchboard_model::bytes::Base64Bytes(cert_bytes)],
            key: switchboard_model::bytes::Base64Bytes(key_bytes),
            ocsp: None,
        })
    }
}
