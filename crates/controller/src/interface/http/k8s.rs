use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

use axum::{extract::State, response::Response};
use k8s_openapi::api::core::v1::Namespace;
use kube::Api;
use tokio::sync::RwLock;

use super::HttpState;
use crate::utils::k8s;
const NAMESPACE_CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, serde::Serialize)]
pub struct K8sEnvResponse {
    pub in_cluster: bool,
    pub current_namespace: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct K8sNamespacesResponse {
    pub namespaces: Vec<String>,
}

#[derive(Debug, Clone)]
struct NamespaceCacheEntry {
    updated_at: Instant,
    namespaces: Vec<String>,
}

static NAMESPACE_CACHE: OnceLock<RwLock<Option<NamespaceCacheEntry>>> = OnceLock::new();

fn namespace_cache() -> &'static RwLock<Option<NamespaceCacheEntry>> {
    NAMESPACE_CACHE.get_or_init(|| RwLock::new(None))
}

pub async fn get_k8s_env() -> Response {
    let process = async {
        let current_namespace = k8s::current_namespace().await?;
        Ok::<_, crate::Error>(K8sEnvResponse {
            in_cluster: current_namespace.is_some(),
            current_namespace,
        })
    };
    super::result_to_json_response(process.await)
}

pub async fn get_k8s_namespaces(State(_state): State<HttpState>) -> Response {
    let process = async {
        let Some(client) = k8s::kube_client_if_in_cluster().await? else {
            return Err(crate::Error::NotInKubernetesCluster);
        };

        if let Some(namespaces) = read_namespace_cache().await {
            return Ok::<_, crate::Error>(K8sNamespacesResponse { namespaces });
        }

        let namespace_api: Api<Namespace> = Api::all(client);
        let namespace_list = namespace_api.list(&Default::default()).await?;
        let mut namespaces = namespace_list
            .items
            .into_iter()
            .filter_map(|namespace| namespace.metadata.name)
            .collect::<Vec<_>>();
        namespaces.sort();
        namespaces.dedup();

        write_namespace_cache(namespaces.clone()).await;

        Ok::<_, crate::Error>(K8sNamespacesResponse { namespaces })
    };
    super::result_to_json_response(process.await)
}

async fn read_namespace_cache() -> Option<Vec<String>> {
    let cache = namespace_cache();
    let guard = cache.read().await;
    let Some(entry) = guard.as_ref() else {
        return None;
    };
    if entry.updated_at.elapsed() > NAMESPACE_CACHE_TTL {
        return None;
    }
    Some(entry.namespaces.clone())
}

async fn write_namespace_cache(namespaces: Vec<String>) {
    let cache = namespace_cache();
    let mut guard = cache.write().await;
    *guard = Some(NamespaceCacheEntry {
        updated_at: Instant::now(),
        namespaces,
    });
}

pub fn router() -> axum::Router<HttpState> {
    axum::Router::new()
        .route("/env", axum::routing::get(get_k8s_env))
        .route("/namespaces", axum::routing::get(get_k8s_namespaces))
}
