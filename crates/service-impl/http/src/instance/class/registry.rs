use crate::flow::{
    filter::{AsFilterClass, FilterClass},
    node::{AsNodeClass, NodeClass},
};
use crate::instance::{class::*, InstanceValue};
use std::collections::HashMap;
use switchboard_service::SerdeValue;

#[derive(Clone)]
pub struct ClassDataWithConstructor {
    pub data: ClassData,
    pub constructor: Constructor,
}

impl std::fmt::Debug for ClassDataWithConstructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassDataWithConstructor")
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClassRegistry {
    pub class_data: HashMap<ClassId, ClassDataWithConstructor>,
}
#[derive(Debug, thiserror::Error)]
pub enum ClassRegistryError {
    #[error("Construct Error: {0}")]
    ConstructError(#[from] ConstructError),
    #[error("Class `{id}` not found")]
    ClassNotFound { id: ClassId },
}

impl ClassRegistry {
    pub fn const_new() -> Self {
        Self {
            class_data: HashMap::new(),
        }
    }
    pub fn construct(
        &self,
        class_id: ClassId,
        config: SerdeValue,
    ) -> Result<InstanceValue, ClassRegistryError> {
        let class_data = self
            .class_data
            .get(&class_id)
            .ok_or_else(|| ClassRegistryError::ClassNotFound { id: class_id })?;
        class_data
            .constructor
            .construct(&config)
            .map_err(ClassRegistryError::ConstructError)
    }
    pub fn register<C: Class>(&mut self, class: C) {
        let class_id = class.id();
        let class_data = ClassData {
            id: class_id.clone(),
            meta: class.meta(),
            instance_type: class.instance_type(),
            // config_schema: class.schema(),
        };
        self.class_data.insert(
            class_id,
            ClassDataWithConstructor {
                data: class_data,
                constructor: Constructor::from_class(class),
            },
        );
    }
    pub fn register_node<N: NodeClass>(&mut self, class: N) {
        self.register(AsNodeClass(class));
    }
    pub fn register_filter<F: FilterClass>(&mut self, class: F) {
        self.register(AsFilterClass(class));
    }
}

#[cfg(feature = "service-impl")]
mod resigter {
    use std::sync::{Arc, OnceLock};
    use tokio::sync::RwLock;
    static GLOBAL_CLASS_REGISTRY: OnceLock<Arc<RwLock<super::ClassRegistry>>> = OnceLock::new();
    use crate::{
        flow::{
            balancer::BalancerClass,
            filter::{
                request_header_modify::RequestHeaderModifyFilterClass,
                request_mirror::RequestMirrorFilterClass,
                request_rate_limit::RequestRateLimitFilterClass,
                request_redirect::RequestRedirectFilterClass,
                response_header_modify::ResponseHeaderModifyFilterClass, timeout::Timeout,
                url_rewrite::UrlRewriteFilterClass,
            },
            router::router::RouterRouterClass,
            service::{
                http_client::HttpClientClass, reverse_proxy::ReverseProxyServiceClass,
                static_file::StaticFileClass, static_response::StaticResponseServiceClass,
            },
        },
        HttpProvider,
    };
    impl super::ClassRegistry {
        pub fn register_prelude(&mut self) {
            // nodes
            {
                // balancers
                self.register_node(BalancerClass);

                // routers
                self.register_node(RouterRouterClass);

                // services
                self.register_node(HttpClientClass);
                self.register_node(StaticResponseServiceClass);
                self.register_node(ReverseProxyServiceClass);
                self.register_node(StaticFileClass);
            }
            // filters
            {
                self.register_filter(UrlRewriteFilterClass);
                self.register_filter(RequestMirrorFilterClass);
                self.register_filter(RequestRateLimitFilterClass);
                self.register_filter(RequestHeaderModifyFilterClass);
                self.register_filter(RequestRedirectFilterClass);
                self.register_filter(ResponseHeaderModifyFilterClass);
                self.register_filter(Timeout);
            }
        }
        pub fn global(provider: &HttpProvider) -> Arc<RwLock<Self>> {
            GLOBAL_CLASS_REGISTRY
                .get_or_init(|| {
                    let mut registry = Self::default();
                    registry.register_prelude();
                    // loading dynamic libs
                    for lib in &provider.rust_dyn_libs {
                        let _ = registry
                            .load_dynamic_lib(lib)
                            .inspect_err(|e| tracing::error!("fail to load dyn lib: {e}"));
                    }
                    Arc::new(RwLock::new(registry))
                })
                .clone()
        }
    }
}
