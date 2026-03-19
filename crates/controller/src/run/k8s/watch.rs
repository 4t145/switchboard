use std::fmt::Debug;

use futures::StreamExt;
use kube::runtime::watcher::{self, Event};
use kube::{Api, Client, Resource, ResourceExt};
use serde::de::DeserializeOwned;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::{ChangeKind, K8sRuntimeEvent, ObjectKey, ResourceKind};

pub mod core;
pub mod gateway_api;

const RESTART_ALL_OBJECTS_KEY: &str = "*";

#[derive(Clone)]
pub struct ResourceWatcherContext {
    pub ct: CancellationToken,
    pub tx: mpsc::Sender<K8sRuntimeEvent>,
}

impl ResourceWatcherContext {
    pub fn new(ct: CancellationToken, tx: mpsc::Sender<K8sRuntimeEvent>) -> Self {
        Self { ct, tx }
    }
}

pub fn spawn_all_watchers(client: Client, context: ResourceWatcherContext) -> Vec<JoinHandle<()>> {
    let mut handles = gateway_api::spawn_watchers(client.clone(), context.clone());
    handles.extend(core::spawn_watchers(client, context));
    handles
}

pub(super) fn spawn_resource_watcher<K>(
    api: Api<K>,
    context: ResourceWatcherContext,
    resource: ResourceKind,
) -> JoinHandle<()>
where
    K: Clone + Debug + DeserializeOwned + ResourceExt + Resource + Send + 'static,
    <K as Resource>::DynamicType: Default + Send + Sync + 'static,
{
    tokio::spawn(async move {
        let mut stream = watcher::watcher(api, watcher::Config::default()).boxed();

        loop {
            let next = tokio::select! {
                _ = context.ct.cancelled() => {
                    tracing::debug!(resource = ?resource, "watcher cancelled");
                    break;
                }
                next = stream.next() => next,
            };

            let Some(next) = next else {
                tracing::debug!(resource = ?resource, "watcher stream finished");
                break;
            };

            match next {
                Ok(event) => {
                    handle_watcher_event(&context.tx, resource, event).await;
                }
                Err(err) => {
                    let message = err.to_string();
                    tracing::warn!(resource = ?resource, error = %message, "watcher stream error");
                    let send_result = context
                        .tx
                        .send(K8sRuntimeEvent::WatcherError { resource, message })
                        .await;
                    if send_result.is_err() {
                        break;
                    }
                }
            }
        }
    })
}

async fn handle_watcher_event<K>(
    tx: &mpsc::Sender<K8sRuntimeEvent>,
    resource: ResourceKind,
    event: Event<K>,
) where
    K: ResourceExt,
{
    match event {
        Event::Apply(resource_obj) | Event::InitApply(resource_obj) => {
            let key = ObjectKey::from_resource(&resource_obj);
            let _ = tx
                .send(K8sRuntimeEvent::ResourceChanged {
                    resource,
                    change: ChangeKind::Applied,
                    key,
                })
                .await;
        }
        Event::Delete(resource_obj) => {
            let key = ObjectKey::from_resource(&resource_obj);
            let _ = tx
                .send(K8sRuntimeEvent::ResourceChanged {
                    resource,
                    change: ChangeKind::Deleted,
                    key,
                })
                .await;
        }
        Event::Init => {
            let _ = tx
                .send(K8sRuntimeEvent::ResourceChanged {
                    resource,
                    change: ChangeKind::Restarted,
                    key: ObjectKey {
                        namespace: None,
                        name: RESTART_ALL_OBJECTS_KEY.to_string(),
                        uid: None,
                        generation: None,
                    },
                })
                .await;
        }
        Event::InitDone => {}
    }
}
