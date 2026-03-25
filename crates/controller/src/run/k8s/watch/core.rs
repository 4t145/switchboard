use k8s_openapi::api::core::v1::{Namespace, Secret, Service};
use k8s_openapi::api::discovery::v1::EndpointSlice;
use kube::{Api, Client};
use tokio::task::JoinHandle;

use super::{ResourceWatcherContext, spawn_resource_watcher};
use crate::run::k8s::ResourceKind;

pub fn spawn_watchers(client: Client, context: ResourceWatcherContext) -> Vec<JoinHandle<()>> {
    vec![
        spawn_resource_watcher::<Service>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::Service,
        ),
        spawn_resource_watcher::<EndpointSlice>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::EndpointSlice,
        ),
        spawn_resource_watcher::<Secret>(
            Api::all(client.clone()),
            context.clone(),
            ResourceKind::Secret,
        ),
        spawn_resource_watcher::<Namespace>(Api::all(client), context, ResourceKind::Namespace),
    ]
}
