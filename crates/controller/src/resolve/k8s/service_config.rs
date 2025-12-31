use std::{
    collections::{BTreeMap, HashMap},
    net::{Ipv4Addr, SocketAddr},
};

use k8s_openapi::api::core::v1::Secret;
use kube::{Api, ResourceExt, api::ListParams, client};
use serde::{Deserialize, Serialize};
use switchboard_custom_config::{
    K8sResource, SerdeValue,
    switchboard_serde_value::{self, value},
};

use switchboard_model::{
    Listener, TcpServiceConfig, Tls, TlsCertParams,
    services::{
        self,
        http::{
            InstanceData, InstanceId, NodeOutput, NodePort, NodeTarget, consts::ROUTER_CLASS_ID,
        },
    },
    tcp_route::TcpRoute,
};

use crate::resolve::k8s::K8sServiceConfigResolver;

mod backend;
mod filter;
mod route;
mod rule_match;
fn target_name(name: &str, namespace: Option<&str>, port: Option<u16>) -> String {
    match (namespace, port) {
        (Some(ns), Some(p)) => format!("{}.{}.port{}", name, ns, p),
        (Some(ns), None) => format!("{}.{}", name, ns),
        (None, Some(p)) => format!("{}.port{}", name, p),
        (None, None) => name.to_string(),
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct K8sServiceBuildConfig {
    pub gateway_namespace: String,
}

impl Default for K8sServiceBuildConfig {
    fn default() -> Self {
        Self {
            gateway_namespace: "default".to_string(),
        }
    }
}
#[derive(Debug, Clone, Default)]
struct K8sGatewayGatheredResource {
    pub gateway_classes: BTreeMap<String, K8sGateways>,
}

#[derive(Debug, Clone, Default)]
struct K8sGateways {
    pub gateways: BTreeMap<String, K8sGatewayGatewayData>,
}

struct ServiceBuilder {
    pub config: switchboard_model::ServiceConfig<SerdeValue, K8sResource>,
    pub client: kube::Client,
}
#[derive(thiserror::Error, Debug)]
enum ServiceBuilderError {
    #[error("HTTP Router build error: {0}")]
    HttpRouterBuildError(#[from] switchboard_http_router::error::BuildError),
    #[error("Serde value error: {0}")]
    SerdeValueError(#[from] switchboard_serde_value::Error),
    #[error("k8s error {0}")]
    K8sError(#[from] kube::Error),
    #[error("tls cert params error {0}")]
    TlsCertParamsError(#[from] switchboard_model::tls::TlsCertParamsError),
}
impl ServiceBuilder {
    pub fn new(client: kube::Client) -> Self {
        Self {
            config: switchboard_model::ServiceConfig::<SerdeValue, K8sResource>::default(),
            client,
        }
    }
    pub async fn fetch_tls_cert_params(
        client: kube::Client,
        resource: &K8sResource,
    ) -> Result<TlsCertParams, ServiceBuilderError> {
        let secrets: Api<Secret> = if let Some(ns) = &resource.namespace {
            Api::namespaced(client.clone(), ns)
        } else {
            Api::default_namespaced(client.clone())
        };
        let secret = secrets.get(&resource.name).await?;
        let data = secret.data.unwrap_or_default();
        let cert_bytes = data.get("tls.crt").cloned().unwrap_or_default().0;
        let key_bytes = data.get("tls.key").cloned().unwrap_or_default().0;
        let tls_cert_params =
            switchboard_model::tls::TlsCertParams::from_bytes(&cert_bytes, &key_bytes)?;

        Ok(tls_cert_params)
    }
    pub async fn resolve(self) -> Result<switchboard_model::ServiceConfig, ServiceBuilderError> {
        let switchboard_model::ServiceConfig::<SerdeValue, K8sResource> {
            tcp_services,
            tcp_listeners,
            tcp_routes,
            tls,
        } = self.config;
        let mut resolved_tls = BTreeMap::new();
        for (name, tls_link) in tls {
            let Tls {
                resolver: resource,
                options,
            } = tls_link;
            let tls_param = Self::fetch_tls_cert_params(self.client.clone(), &resource).await?;
            let resolved_tls_resolver = tls_param.into();
            resolved_tls.insert(
                name,
                Tls {
                    resolver: resolved_tls_resolver,
                    options,
                },
            );
        }
        let resolved_config = switchboard_model::ServiceConfig {
            tcp_services,
            tcp_listeners,
            tcp_routes,
            tls: resolved_tls,
        };
        Ok(resolved_config)
    }
    pub fn build_http_service(
        &mut self,
        gateway: &K8sGatewayGatewayData,
    ) -> Result<String, ServiceBuilderError> {
        let gateway_name = gateway.gateway.name_any();
        let config = HttpGatewayBuilder::new().build(&gateway.http_routes)?;
        let value = switchboard_serde_value::SerdeValue::serialize_from(&config)?;
        tracing::debug!(
            "Built HTTP Gateway service for K8s Gateway {}: {:?}",
            gateway_name,
            value
        );
        let service_name = format!("http-gateway-{}", gateway.gateway.name_any());
        let service_instance = TcpServiceConfig {
            provider: "http".to_string(),
            name: service_name.clone(),
            config: Some(value),
            description: Some(format!(
                "HTTP Gateway for K8s Gateway {}",
                gateway.gateway.name_any()
            )),
        };
        for k8s_listener in &gateway.gateway.spec.listeners {
            let port = k8s_listener.port as u16;
            let bind = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
            let listener = Listener {
                bind,
                description: Some(format!("listener on {port} for {gateway_name}")),
            };
            let mut route = TcpRoute {
                bind,
                service: service_name.clone(),
                tls: None,
            };
            if let Some(tls) = &k8s_listener.tls
                && let Some(listener_certificate_refs) = tls.certificate_refs.as_ref()
            {
                for listener_certificate_ref in listener_certificate_refs {
                    let tls_resource = K8sResource::new(
                        listener_certificate_ref.namespace.clone(),
                        listener_certificate_ref.name.clone(),
                    );
                    let tls_name = format!("k8s-gateway-{}-listener-{}-tls", gateway_name, port);
                    self.config.tls.insert(
                        tls_name.clone(),
                        Tls {
                            resolver: tls_resource,
                            options: Default::default(),
                        },
                    );
                    route.tls = Some(tls_name)
                }
            }
            self.config.tcp_listeners.insert(bind, listener);
            self.config.tcp_routes.insert(bind, route);
        }
        self.config
            .tcp_services
            .insert(service_name.clone(), service_instance);
        Ok(service_name)
    }
}

#[derive(Debug, Clone, Default)]
struct HttpBuildingRouter {
    outputs: BTreeMap<NodePort, NodeOutput>,
    router: switchboard_http_router::serde::RouterSerde<NodePort>,
}

struct HttpGatewayBuilder {
    pub config: services::http::Config,
    pub internal_error_500_page_instance_id: InstanceId,
}

impl HttpGatewayBuilder {
    pub fn new() -> Self {
        Self {
            config: services::http::Config::default(),
            internal_error_500_page_instance_id: InstanceId::new("internal-error-500-page"),
        }
    }

    fn build_router(
        &mut self,
        http_routes: &BTreeMap<String, gateway_api::httproutes::HTTPRoute>,
    ) -> Result<(), switchboard_http_router::error::BuildError> {
        let mut building_router = HttpBuildingRouter::default();
        // let listener = gateway.gateway.spec.listeners;
        for (route_name, route) in http_routes {
            self.build_route(route_name, route, &mut building_router);
        }
        let config = value!({
            "outputs": building_router.outputs,
            "hostname": building_router.router.hostname,
        });
        let router_instance = InstanceData {
            class: services::http::ClassId::std(ROUTER_CLASS_ID),
            name: None,
            r#type: services::http::InstanceType::Node,
            config,
        };
        let router_id = InstanceId::new("router");
        self.config.flow.entrypoint = NodeTarget {
            id: router_id.clone(),
            port: NodePort::Default,
        };
        self.config
            .flow
            .instances
            .insert(router_id, router_instance);
        Ok(())
    }

    pub fn build(
        mut self,
        http_routes: &BTreeMap<String, gateway_api::httproutes::HTTPRoute>,
    ) -> Result<services::http::Config, switchboard_http_router::error::BuildError> {
        self.build_router(http_routes)?;
        Ok(self.config)
    }
}

#[derive(Debug, Clone, Default)]
struct K8sGatewayGatewayData {
    pub gateway: gateway_api::gateways::Gateway,
    pub http_routes: BTreeMap<String, gateway_api::httproutes::HTTPRoute>,
}

// pub struct K8sGatewayHttpRouteData {
//     pub http_route: gateway_api::httproutes::HTTPRoute,
// }

#[derive(thiserror::Error, Debug)]
pub enum K8sGatewayResourceError {
    #[error("No Kubernetes client available")]
    NoK8sClient,
    #[error("Kubernetes client error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("Service build error: {0}")]
    ServiceBuilderError(#[from] ServiceBuilderError),
}

pub const GATEWAY_CONTROLLER_NAME: &str = "switchboard.io/gateway-controller";

#[derive(Clone)]
pub struct K8sServiceConfigBuilder {
    pub client: kube::Client,
    pub config: K8sServiceBuildConfig,
}

impl K8sServiceConfigBuilder {
    pub fn new(client: kube::Client, config: K8sServiceBuildConfig) -> Self {
        Self { client, config }
    }
    pub async fn build_config_from_k8s(
        &self,
    ) -> Result<switchboard_model::ServiceConfig, K8sGatewayResourceError> {
        let gateways = self.gather_k8s_gateway_config().await?;
        let mut builder = ServiceBuilder::new(self.client.clone());
        for (_gateway_name, gateway_data) in gateways.gateways {
            builder.build_http_service(&gateway_data)?;
        }
        let config = builder.resolve().await?;
        Ok(config)
    }
    pub(crate) async fn gather_k8s_gateway_config(
        &self,
    ) -> Result<K8sGateways, K8sGatewayResourceError> {
        let mut gathered_gateways = K8sGateways::default();
        let client = self.client.clone();
        let gateway_class_api =
            kube::Api::<gateway_api::gatewayclasses::GatewayClass>::all(client.clone());
        tracing::debug!("Fetching K8s GatewayClasses");
        let gateway_list = gateway_class_api
            .list(&ListParams {
                ..Default::default()
            })
            .await?;
        // filter out only switchboard managed gateway classes
        let gateway_classes = gateway_list
            .items
            .into_iter()
            .filter(|gc| gc.spec.controller_name == GATEWAY_CONTROLLER_NAME)
            .map(|gc| (gc.name_any(), gc))
            .collect::<HashMap<_, _>>();

        tracing::debug!("Found {} GatewayClasses", gateway_classes.len());
        // scan all gateway
        let gateway_api = kube::Api::<gateway_api::gateways::Gateway>::all(client.clone());
        let gateways = gateway_api
            .list(&ListParams {
                ..Default::default()
            })
            .await?;
        // filter out only switchboard managed gateways
        let gateways = gateways
            .items
            .into_iter()
            .filter(|gw| gateway_classes.contains_key(&gw.spec.gateway_class_name))
            .map(|gw| (gw.name_any(), gw))
            .collect::<HashMap<_, _>>();
        tracing::debug!("Found {} Gateways", gateways.len());
        let route_api = kube::Api::<gateway_api::httproutes::HTTPRoute>::namespaced(
            client.clone(),
            &self.config.gateway_namespace,
        );
        let route_list = route_api.list(&ListParams::default()).await?;
        let mut gateway_router_map =
            HashMap::<String, Vec<gateway_api::httproutes::HTTPRoute>>::new();
        for route in route_list.items.into_iter() {
            for pr in route
                .spec
                .parent_refs
                .clone()
                .unwrap_or_default()
                .into_iter()
            {
                let parent_name = pr.name;
                gateway_router_map
                    .entry(parent_name)
                    .or_default()
                    .push(route.clone());
            }
        }
        for (gateway_name, gateway) in gateways {
            tracing::debug!("Processing Gateway: {}", gateway_name);
            let gateway_name = gateway.name_any();
            let mut gateway_data = K8sGatewayGatewayData {
                gateway: gateway.clone(),
                http_routes: BTreeMap::new(),
            };

            for route in gateway_router_map.remove(&gateway_name).unwrap_or_default() {
                let route_name = route.name_any();
                gateway_data.http_routes.insert(route_name, route);
            }
            gathered_gateways
                .gateways
                .insert(gateway_name, gateway_data);
        }
        Ok(gathered_gateways)
    }
}
