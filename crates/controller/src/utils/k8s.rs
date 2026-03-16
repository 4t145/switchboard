use std::io::ErrorKind;
use std::sync::OnceLock;

use tokio::sync::RwLock;

pub const POD_NAMESPACE_ENV: &str = "POD_NAMESPACE";
pub const SERVICE_ACCOUNT_NAMESPACE_PATH: &str =
    "/var/run/secrets/kubernetes.io/serviceaccount/namespace";

#[derive(Debug, thiserror::Error)]
pub enum K8sRuntimeEnvError {
    #[error("failed to read kubernetes namespace file: {0}")]
    ReadNamespaceFile(#[source] std::io::Error),
    #[error("failed to create kubernetes client: {0}")]
    CreateKubeClient(#[source] kube::Error),
}

static CURRENT_NAMESPACE_CACHE: OnceLock<RwLock<Option<Option<String>>>> = OnceLock::new();
static IN_CLUSTER_CLIENT_CACHE: OnceLock<RwLock<Option<Option<kube::Client>>>> = OnceLock::new();

fn current_namespace_cache() -> &'static RwLock<Option<Option<String>>> {
    CURRENT_NAMESPACE_CACHE.get_or_init(|| RwLock::new(None))
}

fn in_cluster_client_cache() -> &'static RwLock<Option<Option<kube::Client>>> {
    IN_CLUSTER_CLIENT_CACHE.get_or_init(|| RwLock::new(None))
}

async fn read_current_namespace_uncached() -> Result<Option<String>, K8sRuntimeEnvError> {
    if let Ok(namespace) = std::env::var(POD_NAMESPACE_ENV) {
        let trimmed = namespace.trim();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed.to_string()));
        }
    }

    match tokio::fs::read_to_string(SERVICE_ACCOUNT_NAMESPACE_PATH).await {
        Ok(namespace_from_file) => {
            let trimmed = namespace_from_file.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(K8sRuntimeEnvError::ReadNamespaceFile(err)),
    }
}

/// Detects current runtime namespace when running inside Kubernetes.
///
/// Returns `Ok(None)` when the process is not running in a Kubernetes pod.
///
/// # Errors
/// Returns [`K8sRuntimeEnvError::ReadNamespaceFile`] when reading the
/// service-account namespace file fails for reasons other than file-not-found.
pub async fn current_namespace() -> Result<Option<String>, K8sRuntimeEnvError> {
    {
        let guard = current_namespace_cache().read().await;
        if let Some(namespace) = guard.as_ref() {
            return Ok(namespace.clone());
        }
    }

    let namespace = read_current_namespace_uncached().await?;
    let mut guard = current_namespace_cache().write().await;
    *guard = Some(namespace.clone());
    Ok(namespace)
}

/// Checks whether current process is running in Kubernetes.
///
/// # Errors
/// Returns any error from [`current_namespace`].
pub async fn in_cluster() -> Result<bool, K8sRuntimeEnvError> {
    Ok(current_namespace().await?.is_some())
}

/// Creates a Kubernetes client only when running in cluster.
///
/// Returns `Ok(None)` when not in a Kubernetes pod.
///
/// # Errors
/// Returns any error from [`current_namespace`] or
/// [`K8sRuntimeEnvError::CreateKubeClient`] when client initialization fails.
pub async fn kube_client_if_in_cluster() -> Result<Option<kube::Client>, K8sRuntimeEnvError> {
    {
        let guard = in_cluster_client_cache().read().await;
        if let Some(client) = guard.as_ref() {
            return Ok(client.clone());
        }
    }

    if current_namespace().await?.is_none() {
        let mut guard = in_cluster_client_cache().write().await;
        *guard = Some(None);
        return Ok(None);
    }

    let client = kube::Client::try_default()
        .await
        .map_err(K8sRuntimeEnvError::CreateKubeClient)?;
    let mut guard = in_cluster_client_cache().write().await;
    *guard = Some(Some(client.clone()));
    Ok(Some(client))
}
