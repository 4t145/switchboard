use kube::ResourceExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    Applied,
    Deleted,
    Restarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    GatewayClass,
    Gateway,
    HTTPRoute,
    GRPCRoute,
    TCPRoute,
    TLSRoute,
    UDPRoute,
    ReferenceGrant,
    Service,
    EndpointSlice,
    Secret,
    Namespace,
    BackendTLSPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectKey {
    pub namespace: Option<String>,
    pub name: String,
    pub uid: Option<String>,
    pub generation: Option<i64>,
}

impl ObjectKey {
    pub fn from_resource<K>(resource: &K) -> Self
    where
        K: ResourceExt,
    {
        let metadata = resource.meta();
        Self {
            namespace: resource.namespace(),
            name: resource.name_any(),
            uid: metadata.uid.clone(),
            generation: metadata.generation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum K8sRuntimeEvent {
    ResourceChanged {
        resource: ResourceKind,
        change: ChangeKind,
        key: ObjectKey,
    },
    WatcherError {
        resource: ResourceKind,
        message: String,
    },
    ApplyStatusChanged,
}
