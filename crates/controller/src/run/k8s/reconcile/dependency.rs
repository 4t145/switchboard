use gateway_api::gateways::Gateway;
use gateway_api::httproutes::HTTPRoute;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::api::discovery::v1::EndpointSlice;
use kube::api::ListParams;
use kube::{Api, ResourceExt};

use crate::ControllerContext;
use crate::utils::k8s::kube_client_if_in_cluster;

use super::{ChangeKind, ObjectKey, gateway, http_route, trace_reconcile_start};

const SERVICE_BACKEND_KIND: &str = "Service";
const DEFAULT_BACKEND_GROUP: &str = "";
const DEFAULT_PARENT_GROUP: &str = "gateway.networking.k8s.io";
const GATEWAY_PARENT_KIND: &str = "Gateway";
const ENDPOINT_SLICE_SERVICE_LABEL: &str = "kubernetes.io/service-name";

#[derive(Debug, thiserror::Error)]
enum DependencyReconcileError {
    #[error("kubernetes runtime environment error: {0}")]
    RuntimeEnv(#[from] crate::utils::k8s::K8sRuntimeEnvError),
    #[error("kubernetes api error: {0}")]
    Kube(#[from] kube::Error),
    #[error("resource key must contain namespace: {0:?}")]
    MissingNamespace(ObjectKey),
}

pub async fn reconcile_service(context: &ControllerContext, change: ChangeKind, key: &ObjectKey) {
    trace_reconcile_start("service dependency", change, key);

    if let Err(err) = reconcile_service_inner(context, key).await {
        tracing::warn!(
            error = %err,
            namespace = key.namespace.as_deref().unwrap_or("<none>"),
            service = %key.name,
            "failed to run service reverse trigger"
        );
    }
}

pub async fn reconcile_endpoint_slice(
    context: &ControllerContext,
    change: ChangeKind,
    key: &ObjectKey,
) {
    trace_reconcile_start("endpointslice dependency", change, key);

    if let Err(err) = reconcile_endpoint_slice_inner(context, change, key).await {
        tracing::warn!(
            error = %err,
            namespace = key.namespace.as_deref().unwrap_or("<none>"),
            endpoint_slice = %key.name,
            "failed to run endpointslice reverse trigger"
        );
    }
}

pub async fn reconcile_secret(context: &ControllerContext, change: ChangeKind, key: &ObjectKey) {
    trace_reconcile_start("secret dependency", change, key);

    if let Err(err) = reconcile_secret_inner(context, change, key).await {
        tracing::warn!(
            error = %err,
            namespace = key.namespace.as_deref().unwrap_or("<none>"),
            secret = %key.name,
            "failed to run secret reverse trigger"
        );
    }
}

pub async fn reconcile_namespace(context: &ControllerContext, change: ChangeKind, key: &ObjectKey) {
    trace_reconcile_start("namespace dependency", change, key);

    if let Err(err) = reconcile_namespace_inner(context, change, key).await {
        tracing::warn!(
            error = %err,
            namespace = key.namespace.as_deref().unwrap_or("<none>"),
            name = %key.name,
            "failed to run namespace reverse trigger"
        );
    }
}

async fn reconcile_service_inner(
    context: &ControllerContext,
    key: &ObjectKey,
) -> Result<(), DependencyReconcileError> {
    let namespace = key
        .namespace
        .clone()
        .ok_or_else(|| DependencyReconcileError::MissingNamespace(key.clone()))?;

    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };

    trigger_routes_for_service(context, client, &namespace, &key.name).await
}

async fn reconcile_endpoint_slice_inner(
    context: &ControllerContext,
    change: ChangeKind,
    key: &ObjectKey,
) -> Result<(), DependencyReconcileError> {
    let namespace = key
        .namespace
        .clone()
        .ok_or_else(|| DependencyReconcileError::MissingNamespace(key.clone()))?;

    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };

    if matches!(change, ChangeKind::Deleted) {
        return trigger_routes_in_namespace(context, client, &namespace).await;
    }

    let endpoint_slice_api: Api<EndpointSlice> = Api::namespaced(client.clone(), &namespace);
    let endpoint_slice = match endpoint_slice_api.get(&key.name).await {
        Ok(endpoint_slice) => endpoint_slice,
        Err(kube::Error::Api(response)) if response.code == 404 => {
            return trigger_routes_in_namespace(context, client, &namespace).await;
        }
        Err(err) => return Err(err.into()),
    };

    let service_name = endpoint_slice
        .metadata
        .labels
        .as_ref()
        .and_then(|labels| labels.get(ENDPOINT_SLICE_SERVICE_LABEL))
        .cloned();

    let Some(service_name) = service_name else {
        return trigger_routes_in_namespace(context, client, &namespace).await;
    };

    trigger_routes_for_service(context, client, &namespace, &service_name).await
}

async fn reconcile_secret_inner(
    context: &ControllerContext,
    change: ChangeKind,
    key: &ObjectKey,
) -> Result<(), DependencyReconcileError> {
    let namespace = key
        .namespace
        .clone()
        .ok_or_else(|| DependencyReconcileError::MissingNamespace(key.clone()))?;

    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };

    if !matches!(change, ChangeKind::Deleted) {
        let secret_api: Api<Secret> = Api::namespaced(client.clone(), &namespace);
        if let Err(kube::Error::Api(response)) = secret_api.get(&key.name).await
            && response.code == 404
        {
            return Ok(());
        }
    }

    let gateway_api: Api<Gateway> = Api::namespaced(client.clone(), &namespace);
    let gateways = gateway_api.list(&ListParams::default()).await?;

    for gateway_resource in gateways.items {
        if !gateway_references_secret(&gateway_resource, &key.name) {
            continue;
        }

        let gateway_key = ObjectKey::from_resource(&gateway_resource);
        gateway::reconcile(context, ChangeKind::Applied, &gateway_key).await;

        let route_api: Api<HTTPRoute> = Api::all(client.clone());
        let routes = route_api.list(&ListParams::default()).await?;
        for route in routes.items {
            if !route_references_gateway_parent(&route, &namespace, &gateway_key.name) {
                continue;
            }
            let route_key = ObjectKey::from_resource(&route);
            http_route::reconcile(context, ChangeKind::Applied, &route_key).await;
        }
    }

    Ok(())
}

async fn reconcile_namespace_inner(
    context: &ControllerContext,
    _change: ChangeKind,
    key: &ObjectKey,
) -> Result<(), DependencyReconcileError> {
    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };

    trigger_routes_in_namespace(context, client, &key.name).await
}

async fn trigger_routes_for_service(
    context: &ControllerContext,
    client: kube::Client,
    service_namespace: &str,
    service_name: &str,
) -> Result<(), DependencyReconcileError> {
    let route_api: Api<HTTPRoute> = Api::all(client);
    let routes = route_api.list(&ListParams::default()).await?;

    for route in routes.items {
        let route_namespace = route.namespace().unwrap_or_default();
        if !route_references_service_backend(
            &route,
            &route_namespace,
            service_namespace,
            service_name,
        ) {
            continue;
        }

        let key = ObjectKey::from_resource(&route);
        http_route::reconcile(context, ChangeKind::Applied, &key).await;
    }

    Ok(())
}

async fn trigger_routes_in_namespace(
    context: &ControllerContext,
    client: kube::Client,
    namespace: &str,
) -> Result<(), DependencyReconcileError> {
    let route_api: Api<HTTPRoute> = Api::namespaced(client, namespace);
    let routes = route_api.list(&ListParams::default()).await?;

    for route in routes.items {
        let key = ObjectKey::from_resource(&route);
        http_route::reconcile(context, ChangeKind::Applied, &key).await;
    }

    Ok(())
}

fn route_references_service_backend(
    route: &HTTPRoute,
    route_namespace: &str,
    service_namespace: &str,
    service_name: &str,
) -> bool {
    let Some(rules) = route.spec.rules.as_ref() else {
        return false;
    };

    for rule in rules {
        let Some(backend_refs) = rule.backend_refs.as_ref() else {
            continue;
        };

        for backend_ref in backend_refs {
            let backend_group = backend_ref
                .group
                .as_deref()
                .unwrap_or(DEFAULT_BACKEND_GROUP);
            if backend_group != DEFAULT_BACKEND_GROUP {
                continue;
            }

            let backend_kind = backend_ref.kind.as_deref().unwrap_or(SERVICE_BACKEND_KIND);
            if backend_kind != SERVICE_BACKEND_KIND {
                continue;
            }

            let backend_namespace = backend_ref.namespace.as_deref().unwrap_or(route_namespace);
            if backend_namespace == service_namespace && backend_ref.name == service_name {
                return true;
            }
        }
    }

    false
}

fn gateway_references_secret(gateway: &Gateway, secret_name: &str) -> bool {
    gateway.spec.listeners.iter().any(|listener| {
        listener
            .tls
            .as_ref()
            .and_then(|tls| tls.certificate_refs.as_ref())
            .is_some_and(|cert_refs| {
                cert_refs.iter().any(|cert_ref| {
                    cert_ref.name == secret_name
                        && cert_ref.kind.as_deref().is_none_or(|kind| kind == "Secret")
                        && cert_ref.group.as_deref().is_none_or(str::is_empty)
                })
            })
    })
}

fn route_references_gateway_parent(
    route: &HTTPRoute,
    gateway_namespace: &str,
    gateway_name: &str,
) -> bool {
    let Some(parent_refs) = route.spec.parent_refs.as_ref() else {
        return false;
    };

    let route_namespace = route.namespace().unwrap_or_default();
    parent_refs.iter().any(|parent_ref| {
        if parent_ref.name != gateway_name {
            return false;
        }

        let parent_group = parent_ref.group.as_deref().unwrap_or(DEFAULT_PARENT_GROUP);
        if parent_group != DEFAULT_PARENT_GROUP {
            return false;
        }

        let parent_kind = parent_ref.kind.as_deref().unwrap_or(GATEWAY_PARENT_KIND);
        if parent_kind != GATEWAY_PARENT_KIND {
            return false;
        }

        let parent_namespace = parent_ref.namespace.as_deref().unwrap_or(&route_namespace);
        parent_namespace == gateway_namespace
    })
}
