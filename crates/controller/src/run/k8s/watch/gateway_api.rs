use gateway_api::backendtlspolicies::BackendTLSPolicy;
use gateway_api::experimental::tcproutes::TCPRoute;
use gateway_api::experimental::tlsroutes::TLSRoute;
use gateway_api::experimental::udproutes::UDPRoute;
use gateway_api::gatewayclasses::GatewayClass;
use gateway_api::gateways::Gateway;
use gateway_api::grpcroutes::GRPCRoute;
use gateway_api::httproutes::HTTPRoute;
use gateway_api::referencegrants::ReferenceGrant;
use kube::{Api, Client};
use tokio::task::JoinHandle;

use super::{ResourceWatcherContext, spawn_resource_watcher};
use crate::run::k8s::ResourceKind;

pub fn spawn_watchers(client: Client, context: ResourceWatcherContext) -> Vec<JoinHandle<()>> {
    vec![
        spawn_resource_watcher::<GatewayClass>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::GatewayClass,
        ),
        spawn_resource_watcher::<Gateway>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::Gateway,
        ),
        spawn_resource_watcher::<HTTPRoute>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::HTTPRoute,
        ),
        spawn_resource_watcher::<GRPCRoute>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::GRPCRoute,
        ),
        spawn_resource_watcher::<TCPRoute>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::TCPRoute,
        ),
        spawn_resource_watcher::<TLSRoute>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::TLSRoute,
        ),
        spawn_resource_watcher::<UDPRoute>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::UDPRoute,
        ),
        spawn_resource_watcher::<ReferenceGrant>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::ReferenceGrant,
        ),
        spawn_resource_watcher::<BackendTLSPolicy>(
            Api::all(client),
            context,
            ResourceKind::BackendTLSPolicy,
        ),
    ]
}
