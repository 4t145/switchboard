use crate::ControllerContext;

use super::{ChangeKind, K8sRuntimeEvent, ObjectKey, ResourceKind};

mod backend_tls_policy;
mod dependency;
mod gateway;
mod gateway_class;
mod grpc_route;
mod http_route;
mod reference_grant;
mod tcp_route;
mod tls_route;
mod udp_route;

pub async fn dispatch_event(context: &ControllerContext, event: K8sRuntimeEvent) {
    match event {
        K8sRuntimeEvent::ResourceChanged {
            resource,
            change,
            key,
        } => {
            tracing::debug!(
                resource = ?resource,
                change = ?change,
                namespace = key.namespace.as_deref().unwrap_or("<cluster>"),
                name = %key.name,
                uid = key.uid.as_deref().unwrap_or("<none>"),
                generation = key.generation.unwrap_or_default(),
                "received k8s resource event"
            );

            match resource {
                ResourceKind::GatewayClass => gateway_class::reconcile(context, change, &key).await,
                ResourceKind::Gateway => gateway::reconcile(context, change, &key).await,
                ResourceKind::HTTPRoute => http_route::reconcile(context, change, &key).await,
                ResourceKind::GRPCRoute => grpc_route::reconcile(context, change, &key).await,
                ResourceKind::TCPRoute => tcp_route::reconcile(context, change, &key).await,
                ResourceKind::TLSRoute => tls_route::reconcile(context, change, &key).await,
                ResourceKind::UDPRoute => udp_route::reconcile(context, change, &key).await,
                ResourceKind::ReferenceGrant => {
                    reference_grant::reconcile(context, change, &key).await
                }
                ResourceKind::Service => dependency::reconcile_service(context, change, &key).await,
                ResourceKind::EndpointSlice => {
                    dependency::reconcile_endpoint_slice(context, change, &key).await
                }
                ResourceKind::Secret => dependency::reconcile_secret(context, change, &key).await,
                ResourceKind::Namespace => {
                    dependency::reconcile_namespace(context, change, &key).await
                }
                ResourceKind::BackendTLSPolicy => {
                    backend_tls_policy::reconcile(context, change, &key).await
                }
            }
        }
        K8sRuntimeEvent::WatcherError { resource, message } => {
            tracing::warn!(resource = ?resource, error = %message, "k8s watcher error event");
        }
        K8sRuntimeEvent::ApplyStatusChanged => {
            gateway::reconcile_programmed_from_apply_status(context).await;
            http_route::reconcile_programmed_from_apply_status(context).await;
        }
    }
}

fn trace_reconcile_start(resource: &'static str, change: ChangeKind, key: &ObjectKey) {
    tracing::trace!(resource, ?change, key = ?key, "reconcile requested");
}
