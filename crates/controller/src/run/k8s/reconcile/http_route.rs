use std::collections::BTreeMap;

use gateway_api::constants::GatewayConditionType as GatewayConditionTypeEnum;
use gateway_api::gateways::{
    Gateway, GatewayListeners, GatewayListenersAllowedRoutesNamespacesFrom,
};
use gateway_api::httproutes::{
    HTTPRoute, HTTPRouteStatus, HTTPRouteStatusParents, HTTPRouteStatusParentsParentRef,
};
use gateway_api::referencegrants::ReferenceGrant;
use k8s_openapi::api::core::v1::{Namespace, Service};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
use k8s_openapi::chrono::Utc;
use kube::api::{ListParams, Patch, PatchParams};
use kube::{Api, ResourceExt};

use crate::ControllerContext;
use crate::run::k8s::ChangeKind;
use crate::utils::k8s::{CONTROLLER_NAME, kube_client_if_in_cluster};

use super::{ObjectKey, trace_reconcile_start};

const CONDITION_TYPE_ACCEPTED: &str = "Accepted";
const CONDITION_TYPE_PROGRAMMED: &str = "Programmed";
const CONDITION_TYPE_RESOLVED_REFS: &str = "ResolvedRefs";

const REASON_ACCEPTED: &str = "Accepted";
const REASON_NO_MATCHING_PARENT: &str = "NoMatchingParent";
const REASON_INVALID_KIND: &str = "InvalidKind";
const REASON_INVALID_BACKEND_REFS: &str = "InvalidBackendRefs";
const REASON_REF_NOT_PERMITTED: &str = "RefNotPermitted";
const REASON_RESOLVED_REFS: &str = "ResolvedRefs";
const REASON_PROGRAMMED: &str = "Programmed";

const STATUS_TRUE: &str = "True";
const STATUS_FALSE: &str = "False";

const DEFAULT_PARENT_GROUP: &str = "gateway.networking.k8s.io";
const GATEWAY_PARENT_KIND: &str = "Gateway";
const HTTP_ROUTE_KIND: &str = "HTTPRoute";
const SERVICE_BACKEND_KIND: &str = "Service";

const MSG_ACCEPTED: &str = "HTTPRoute is accepted by switchboard controller";
const MSG_NO_MATCHING_PARENT: &str =
    "No matching managed Gateway parent found for this HTTPRoute";
const MSG_INVALID_PARENT_KIND: &str =
    "HTTPRoute parentRef kind or group is not supported by switchboard controller";
const MSG_PARENT_NOT_ALLOWED: &str =
    "HTTPRoute is not permitted to attach to referenced Gateway listeners";
const MSG_RESOLVED_REFS: &str = "All backend references are resolved";
const MSG_PROGRAMMED: &str = "HTTPRoute is programmed by switchboard controller";
const MSG_INVALID_BACKEND_REFS: &str = "At least one backendRef is invalid or unresolved";
const MSG_BACKEND_REF_NOT_PERMITTED: &str =
    "At least one cross-namespace backendRef is not permitted by ReferenceGrant";
const MSG_PROGRAMMED_APPLY_FAILED: &str =
    "HTTPRoute is accepted but latest gateway config apply failed";

#[derive(Debug, thiserror::Error)]
pub enum HTTPRouteReconcileError {
    #[error("kubernetes runtime environment error: {0}")]
    RuntimeEnv(#[from] crate::utils::k8s::K8sRuntimeEnvError),
    #[error("kubernetes api error: {0}")]
    Kube(#[from] kube::Error),
    #[error("httproute key must contain namespace: {0:?}")]
    MissingNamespace(ObjectKey),
}

enum RouteParentState {
    Accepted,
    NoMatchingParent,
    InvalidParentKind,
    ParentNotAllowed,
}

enum BackendRefsState {
    Resolved,
    Invalid,
    RefNotPermitted,
}

pub async fn reconcile(context: &ControllerContext, change: ChangeKind, key: &ObjectKey) {
    trace_reconcile_start("httproute", change, key);

    if let Err(err) = reconcile_inner(context, change, key).await {
        tracing::warn!(
            error = %err,
            namespace = key.namespace.as_deref().unwrap_or("<none>"),
            httproute = %key.name,
            "failed to reconcile httproute"
        );
    }
}

async fn reconcile_inner(
    context: &ControllerContext,
    change: ChangeKind,
    key: &ObjectKey,
) -> Result<(), HTTPRouteReconcileError> {
    let namespace = key
        .namespace
        .clone()
        .ok_or_else(|| HTTPRouteReconcileError::MissingNamespace(key.clone()))?;

    if matches!(change, ChangeKind::Deleted) {
        return Ok(());
    }

    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };

    let route_api: Api<HTTPRoute> = Api::namespaced(client.clone(), &namespace);
    let route = match route_api.get(&key.name).await {
        Ok(route) => route,
        Err(kube::Error::Api(response)) if response.code == 404 => return Ok(()),
        Err(err) => return Err(err.into()),
    };

    let generation = route.metadata.generation.unwrap_or(0);
    let apply_status = context.k8s_apply_status.read().await.clone();
    let parents = build_parent_statuses(client, &route, &namespace, generation, apply_status.as_ref()).await?;

    let desired_status = HTTPRouteStatus { parents };
    let status_unchanged = route
        .status
        .as_ref()
        .is_some_and(|status| status == &desired_status);
    if status_unchanged {
        return Ok(());
    }

    let patch = Patch::Merge(serde_json::json!({
        "status": desired_status,
    }));
    route_api
        .patch_status(&key.name, &PatchParams::default(), &patch)
        .await?;

    Ok(())
}

async fn build_parent_statuses(
    client: kube::Client,
    route: &HTTPRoute,
    route_namespace: &str,
    generation: i64,
    apply_status: Option<&crate::run::k8s::K8sApplyStatus>,
) -> Result<Vec<HTTPRouteStatusParents>, HTTPRouteReconcileError> {
    let Some(parent_refs) = route.spec.parent_refs.as_ref() else {
        return Ok(Vec::new());
    };

    let mut parents = Vec::with_capacity(parent_refs.len());
    for parent_ref in parent_refs {
        let parent_ref_status = HTTPRouteStatusParentsParentRef {
            group: parent_ref.group.clone(),
            kind: parent_ref.kind.clone(),
            name: parent_ref.name.clone(),
            namespace: parent_ref.namespace.clone(),
            port: parent_ref.port,
            section_name: parent_ref.section_name.clone(),
        };

        let parent_state = resolve_parent_state(client.clone(), route_namespace, parent_ref).await?;
        let conditions = match parent_state {
            RouteParentState::Accepted => {
                let backend_state =
                    resolve_backend_refs_state(client.clone(), route, route_namespace).await?;
                build_conditions_for_backend_state(generation, backend_state, apply_status)
            }
            RouteParentState::NoMatchingParent => build_conditions_for_parent_state(
                generation,
                REASON_NO_MATCHING_PARENT,
                MSG_NO_MATCHING_PARENT,
            ),
            RouteParentState::InvalidParentKind => {
                build_conditions_for_parent_state(generation, REASON_INVALID_KIND, MSG_INVALID_PARENT_KIND)
            }
            RouteParentState::ParentNotAllowed => build_conditions_for_parent_state(
                generation,
                REASON_REF_NOT_PERMITTED,
                MSG_PARENT_NOT_ALLOWED,
            ),
        };

        parents.push(HTTPRouteStatusParents {
            controller_name: CONTROLLER_NAME.to_string(),
            parent_ref: parent_ref_status,
            conditions,
        });
    }

    Ok(parents)
}

async fn resolve_parent_state(
    client: kube::Client,
    route_namespace: &str,
    parent_ref: &gateway_api::httproutes::HTTPRouteParentRefs,
) -> Result<RouteParentState, HTTPRouteReconcileError> {
    let parent_kind = parent_ref.kind.as_deref().unwrap_or(GATEWAY_PARENT_KIND);
    if parent_kind != GATEWAY_PARENT_KIND {
        return Ok(RouteParentState::InvalidParentKind);
    }

    let parent_group = parent_ref.group.as_deref().unwrap_or(DEFAULT_PARENT_GROUP);
    if parent_group != DEFAULT_PARENT_GROUP {
        return Ok(RouteParentState::InvalidParentKind);
    }

    let gateway_namespace = parent_ref
        .namespace
        .clone()
        .unwrap_or_else(|| route_namespace.to_string());
    let gateway_api: Api<Gateway> = Api::namespaced(client.clone(), &gateway_namespace);
    let gateway = match gateway_api.get(&parent_ref.name).await {
        Ok(gateway) => gateway,
        Err(kube::Error::Api(response)) if response.code == 404 => {
            return Ok(RouteParentState::NoMatchingParent);
        }
        Err(err) => return Err(err.into()),
    };

    let gateway_accepted = gateway
        .status
        .as_ref()
        .and_then(|status| status.conditions.as_ref())
        .and_then(|conditions| {
            conditions.iter().find(|condition| {
                condition.type_ == GatewayConditionTypeEnum::Accepted.to_string()
                    && condition.status == STATUS_TRUE
            })
        })
        .is_some();
    if !gateway_accepted {
        return Ok(RouteParentState::NoMatchingParent);
    }

    if !listener_allows_route(
        client.clone(),
        &gateway,
        route_namespace,
        parent_ref.section_name.as_deref(),
        parent_ref.port,
    )
    .await?
    {
        return Ok(RouteParentState::ParentNotAllowed);
    }

    Ok(RouteParentState::Accepted)
}

async fn listener_allows_route(
    client: kube::Client,
    gateway: &Gateway,
    route_namespace: &str,
    section_name: Option<&str>,
    port: Option<i32>,
) -> Result<bool, HTTPRouteReconcileError> {
    let gateway_namespace = gateway.namespace().unwrap_or_default();
    let listeners = &gateway.spec.listeners;

    let mut namespace_labels: Option<BTreeMap<String, String>> = None;
    for listener in listeners {
        if !listener_matches_parent_ref(listener, section_name, port) {
            continue;
        }
        if !listener_accepts_http_route(listener) {
            continue;
        }

        if listener_namespace_allows_route(
            client.clone(),
            gateway,
            &gateway_namespace,
            route_namespace,
            listener,
            &mut namespace_labels,
        )
        .await?
        {
            return Ok(true);
        }
    }

    Ok(false)
}

fn listener_matches_parent_ref(
    listener: &GatewayListeners,
    section_name: Option<&str>,
    port: Option<i32>,
) -> bool {
    if let Some(section_name) = section_name
        && listener.name != section_name
    {
        return false;
    }

    if let Some(port) = port
        && listener.port != port
    {
        return false;
    }

    true
}

fn listener_accepts_http_route(listener: &GatewayListeners) -> bool {
    let Some(allowed_routes) = listener.allowed_routes.as_ref() else {
        return true;
    };

    let Some(kinds) = allowed_routes.kinds.as_ref() else {
        return true;
    };

    if kinds.is_empty() {
        return true;
    }

    kinds.iter().any(|kind| {
        kind.kind == HTTP_ROUTE_KIND
            && kind
                .group
                .as_deref()
                .is_none_or(|group| group == DEFAULT_PARENT_GROUP)
    })
}

async fn listener_namespace_allows_route(
    client: kube::Client,
    _gateway: &Gateway,
    gateway_namespace: &str,
    route_namespace: &str,
    listener: &GatewayListeners,
    namespace_labels: &mut Option<BTreeMap<String, String>>,
) -> Result<bool, HTTPRouteReconcileError> {
    let from = listener
        .allowed_routes
        .as_ref()
        .and_then(|allowed_routes| allowed_routes.namespaces.as_ref())
        .and_then(|namespaces| namespaces.from.as_ref())
        .unwrap_or(&GatewayListenersAllowedRoutesNamespacesFrom::Same);

    match from {
        GatewayListenersAllowedRoutesNamespacesFrom::All => Ok(true),
        GatewayListenersAllowedRoutesNamespacesFrom::Same => Ok(gateway_namespace == route_namespace),
        GatewayListenersAllowedRoutesNamespacesFrom::Selector => {
            let Some(selector) = listener
                .allowed_routes
                .as_ref()
                .and_then(|allowed_routes| allowed_routes.namespaces.as_ref())
                .and_then(|namespaces| namespaces.selector.as_ref())
            else {
                return Ok(false);
            };

            if namespace_labels.is_none() {
                let namespace_api: Api<Namespace> = Api::all(client);
                let namespace = match namespace_api.get(route_namespace).await {
                    Ok(namespace) => namespace,
                    Err(kube::Error::Api(response)) if response.code == 404 => return Ok(false),
                    Err(err) => return Err(err.into()),
                };
                *namespace_labels = Some(namespace.metadata.labels.unwrap_or_default());
            }

            Ok(namespace_labels
                .as_ref()
                .is_some_and(|labels| matches_namespace_selector(labels, selector)))
        }
    }
}

fn matches_namespace_selector(
    labels: &BTreeMap<String, String>,
    selector: &gateway_api::gateways::GatewayListenersAllowedRoutesNamespacesSelector,
) -> bool {
    if let Some(match_labels) = selector.match_labels.as_ref()
        && match_labels
            .iter()
            .any(|(key, value)| labels.get(key) != Some(value))
    {
        return false;
    }

    if let Some(expressions) = selector.match_expressions.as_ref() {
        for expr in expressions {
            let values = expr.values.as_deref().unwrap_or(&[]);
            let label_value = labels.get(&expr.key);
            let matches = match expr.operator.as_str() {
                "In" => label_value.is_some_and(|value| values.iter().any(|candidate| candidate == value)),
                "NotIn" => label_value.is_some_and(|value| values.iter().all(|candidate| candidate != value)),
                "Exists" => label_value.is_some(),
                "DoesNotExist" => label_value.is_none(),
                _ => false,
            };
            if !matches {
                return false;
            }
        }
    }

    true
}

async fn resolve_backend_refs_state(
    client: kube::Client,
    route: &HTTPRoute,
    route_namespace: &str,
) -> Result<BackendRefsState, HTTPRouteReconcileError> {
    let Some(rules) = route.spec.rules.as_ref() else {
        return Ok(BackendRefsState::Resolved);
    };

    for rule in rules {
        let Some(backend_refs) = rule.backend_refs.as_ref() else {
            continue;
        };
        for backend_ref in backend_refs {
            let backend_group = backend_ref.group.as_deref().unwrap_or("");
            if !backend_group.is_empty() {
                return Ok(BackendRefsState::Invalid);
            }

            let backend_kind = backend_ref.kind.as_deref().unwrap_or(SERVICE_BACKEND_KIND);
            if backend_kind != SERVICE_BACKEND_KIND {
                return Ok(BackendRefsState::Invalid);
            }

            if backend_ref.port.is_none() {
                return Ok(BackendRefsState::Invalid);
            }

            let backend_namespace = backend_ref
                .namespace
                .clone()
                .unwrap_or_else(|| route_namespace.to_string());
            if backend_namespace != route_namespace
                && !reference_grant_allows_backend_ref(
                    client.clone(),
                    &backend_namespace,
                    route_namespace,
                    &backend_ref.name,
                )
                .await?
            {
                return Ok(BackendRefsState::RefNotPermitted);
            }

            let service_api: Api<Service> = Api::namespaced(client.clone(), &backend_namespace);
            let service_exists = match service_api.get(&backend_ref.name).await {
                Ok(_) => true,
                Err(kube::Error::Api(response)) if response.code == 404 => false,
                Err(err) => return Err(err.into()),
            };
            if !service_exists {
                return Ok(BackendRefsState::Invalid);
            }
        }
    }

    Ok(BackendRefsState::Resolved)
}

async fn reference_grant_allows_backend_ref(
    client: kube::Client,
    backend_namespace: &str,
    route_namespace: &str,
    backend_name: &str,
) -> Result<bool, HTTPRouteReconcileError> {
    let reference_grant_api: Api<ReferenceGrant> = Api::namespaced(client, backend_namespace);
    let grants = reference_grant_api.list(&ListParams::default()).await?;

    for grant in grants.items {
        let from_allows = grant.spec.from.iter().any(|from| {
            from.group == DEFAULT_PARENT_GROUP
                && from.kind == HTTP_ROUTE_KIND
                && from.namespace == route_namespace
        });
        if !from_allows {
            continue;
        }

        let to_allows = grant.spec.to.iter().any(|to| {
            to.group.is_empty()
                && to.kind == SERVICE_BACKEND_KIND
                && to
                    .name
                    .as_deref()
                    .is_none_or(|name| name == backend_name)
        });
        if to_allows {
            return Ok(true);
        }
    }

    Ok(false)
}

fn build_conditions_for_parent_state(
    generation: i64,
    reason: &'static str,
    message: &'static str,
) -> Vec<Condition> {
    vec![
        new_condition(
            CONDITION_TYPE_ACCEPTED,
            STATUS_FALSE,
            reason,
            generation,
            message,
        ),
        new_condition(
            CONDITION_TYPE_RESOLVED_REFS,
            STATUS_FALSE,
            reason,
            generation,
            message,
        ),
        new_condition(
            CONDITION_TYPE_PROGRAMMED,
            STATUS_FALSE,
            reason,
            generation,
            message,
        ),
    ]
}

fn build_conditions_for_backend_state(
    generation: i64,
    backend_state: BackendRefsState,
    apply_status: Option<&crate::run::k8s::K8sApplyStatus>,
) -> Vec<Condition> {
    let (resolved_status, resolved_reason, resolved_message, programmed_status, programmed_reason, programmed_message) =
        match backend_state {
            BackendRefsState::Resolved => (
                STATUS_TRUE,
                REASON_RESOLVED_REFS,
                MSG_RESOLVED_REFS,
                STATUS_TRUE,
                REASON_PROGRAMMED,
                MSG_PROGRAMMED,
            ),
            BackendRefsState::Invalid => (
                STATUS_FALSE,
                REASON_INVALID_BACKEND_REFS,
                MSG_INVALID_BACKEND_REFS,
                STATUS_FALSE,
                REASON_INVALID_BACKEND_REFS,
                MSG_INVALID_BACKEND_REFS,
            ),
            BackendRefsState::RefNotPermitted => (
                STATUS_FALSE,
                REASON_REF_NOT_PERMITTED,
                MSG_BACKEND_REF_NOT_PERMITTED,
                STATUS_FALSE,
                REASON_REF_NOT_PERMITTED,
                MSG_BACKEND_REF_NOT_PERMITTED,
            ),
        };

    let programmed = if apply_status.is_some_and(|status| !status.last_apply_succeeded) {
        new_condition(
            CONDITION_TYPE_PROGRAMMED,
            STATUS_FALSE,
            REASON_PROGRAMMED,
            generation,
            MSG_PROGRAMMED_APPLY_FAILED,
        )
    } else {
        new_condition(
            CONDITION_TYPE_PROGRAMMED,
            programmed_status,
            programmed_reason,
            generation,
            programmed_message,
        )
    };

    vec![
        new_condition(
            CONDITION_TYPE_ACCEPTED,
            STATUS_TRUE,
            REASON_ACCEPTED,
            generation,
            MSG_ACCEPTED,
        ),
        new_condition(
            CONDITION_TYPE_RESOLVED_REFS,
            resolved_status,
            resolved_reason,
            generation,
            resolved_message,
        ),
        programmed,
    ]
}

pub async fn reconcile_programmed_from_apply_status(context: &ControllerContext) {
    if let Err(err) = reconcile_programmed_from_apply_status_inner(context).await {
        tracing::warn!(error = %err, "failed to refresh httproute programmed status from apply result");
    }
}

async fn reconcile_programmed_from_apply_status_inner(
    context: &ControllerContext,
) -> Result<(), HTTPRouteReconcileError> {
    let Some(client) = kube_client_if_in_cluster().await? else {
        return Ok(());
    };
    let apply_status = context.k8s_apply_status.read().await.clone();

    let route_api: Api<HTTPRoute> = Api::all(client.clone());
    let routes = route_api.list(&ListParams::default()).await?;
    for route in routes.items {
        let Some(namespace) = route.namespace() else {
            continue;
        };
        let route_api_ns: Api<HTTPRoute> = Api::namespaced(client.clone(), &namespace);
        let generation = route.metadata.generation.unwrap_or(0);
        let parents = build_parent_statuses(
            client.clone(),
            &route,
            &namespace,
            generation,
            apply_status.as_ref(),
        )
        .await?;
        let desired_status = HTTPRouteStatus { parents };
        let status_unchanged = route
            .status
            .as_ref()
            .is_some_and(|status| status == &desired_status);
        if status_unchanged {
            continue;
        }
        let patch = Patch::Merge(serde_json::json!({
            "status": desired_status,
        }));
        route_api_ns
            .patch_status(&route.name_any(), &PatchParams::default(), &patch)
            .await?;
    }

    Ok(())
}

fn new_condition(
    condition_type: &'static str,
    status: &'static str,
    reason: &'static str,
    observed_generation: i64,
    message: &'static str,
) -> Condition {
    Condition {
        type_: condition_type.to_string(),
        status: status.to_string(),
        reason: reason.to_string(),
        observed_generation: Some(observed_generation),
        message: message.to_string(),
        last_transition_time: Time(Utc::now()),
    }
}
