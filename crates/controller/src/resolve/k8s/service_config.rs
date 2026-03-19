use std::collections::{BTreeMap, HashMap};

use k8s_openapi::api::core::v1::Secret;
use kube::{Api, ResourceExt, api::ListParams};
use serde::{Deserialize, Serialize};
use switchboard_model::{
    HumanReadableServiceConfig, SerdeValue, Tls, TlsCertParams, switchboard_serde_value,
};

use crate::{link_resolver::Link, utils::k8s::CONTROLLER_NAME};

mod http;
mod tcp;

struct K8sResource {
    pub namespace: Option<String>,
    pub name: String,
}

impl K8sResource {
    pub fn new(namespace: Option<String>, name: String) -> Self {
        Self { namespace, name }
    }
}

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
struct K8sGateways {
    pub gateways: BTreeMap<String, K8sGatewayGatewayData>,
}

struct ServiceBuilder {
    pub config: switchboard_model::ServiceConfig<SerdeValue, K8sResource>,
    pub client: kube::Client,
}

#[derive(thiserror::Error, Debug)]
pub enum ServiceBuilderError {
    #[error("HTTP Router build error: {0}")]
    HttpRouterBuild(#[from] switchboard_http_router::error::BuildError),
    #[error("Serde value error: {0}")]
    SerdeValue(#[from] switchboard_serde_value::Error),
    #[error("k8s error {0}")]
    K8s(#[from] kube::Error),
    #[error("tls cert params error {0}")]
    TlsCertParams(#[from] switchboard_model::tls::TlsCertParamsError),
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
        let tls_cert_params = switchboard_model::tls::TlsCertParams::from_bytes(&cert_bytes, &key_bytes)?;

        Ok(tls_cert_params)
    }

    pub async fn resolve(self) -> Result<HumanReadableServiceConfig<Link>, ServiceBuilderError> {
        let switchboard_model::ServiceConfig::<SerdeValue, K8sResource> {
            tcp_services,
            tcp_listeners,
            tcp_routes,
            tls,
        } = self.config;
        let mut resolved_tls = BTreeMap::new();
        for (name, tls_link) in tls {
            let Tls { resolver: resource, options } = tls_link;
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
        let resolved_config = HumanReadableServiceConfig::<Link>::from_standard(resolved_config);
        Ok(resolved_config)
    }
}

#[derive(Debug, Clone, Default)]
struct K8sGatewayGatewayData {
    pub gateway: gateway_api::gateways::Gateway,
    pub http_routes: BTreeMap<String, gateway_api::httproutes::HTTPRoute>,
    pub tcp_routes: BTreeMap<String, gateway_api::experimental::tcproutes::TCPRoute>,
    pub tls_routes: BTreeMap<String, gateway_api::experimental::tlsroutes::TLSRoute>,
}

#[derive(thiserror::Error, Debug)]
pub enum K8sGatewayResourceError {
    #[error("No Kubernetes client available")]
    NoK8sClient,
    #[error("Kubernetes client error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("Kubernetes runtime environment error: {0}")]
    RuntimeEnvError(#[from] crate::utils::k8s::K8sRuntimeEnvError),
    #[error("Service build error: {0}")]
    ServiceBuilderError(#[from] ServiceBuilderError),
}

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
    ) -> Result<HumanReadableServiceConfig<Link>, K8sGatewayResourceError> {
        let gateways = self.gather_k8s_gateway_config().await?;
        let mut builder = ServiceBuilder::new(self.client.clone());
        for (_gateway_name, gateway_data) in gateways.gateways {
            let _ = builder.build_http_service(&gateway_data)?;
            let _ = builder.build_tcp_services(&gateway_data)?;
        }
        let config = builder.resolve().await?;
        Ok(config)
    }

    async fn gather_k8s_gateway_config(&self) -> Result<K8sGateways, K8sGatewayResourceError> {
        let mut gathered_gateways = K8sGateways::default();
        let client = self.client.clone();
        let gateway_class_api = kube::Api::<gateway_api::gatewayclasses::GatewayClass>::all(client.clone());
        tracing::debug!("Fetching K8s GatewayClasses");
        let gateway_list = gateway_class_api
            .list(&ListParams { ..Default::default() })
            .await?;
        let gateway_classes = gateway_list
            .items
            .into_iter()
            .filter(|gc| gc.spec.controller_name == CONTROLLER_NAME)
            .map(|gc| (gc.name_any(), gc))
            .collect::<HashMap<_, _>>();

        tracing::debug!("Found {} GatewayClasses", gateway_classes.len());
        let gateway_api = kube::Api::<gateway_api::gateways::Gateway>::all(client.clone());
        let gateways = gateway_api
            .list(&ListParams { ..Default::default() })
            .await?;
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
        let tcp_route_api = kube::Api::<gateway_api::experimental::tcproutes::TCPRoute>::namespaced(
            client.clone(),
            &self.config.gateway_namespace,
        );
        let tls_route_api = kube::Api::<gateway_api::experimental::tlsroutes::TLSRoute>::namespaced(
            client.clone(),
            &self.config.gateway_namespace,
        );
        let route_list = route_api.list(&ListParams::default()).await?;
        let tcp_route_list = tcp_route_api.list(&ListParams::default()).await?;
        let tls_route_list = tls_route_api.list(&ListParams::default()).await?;

        let mut gateway_router_map = HashMap::<String, Vec<gateway_api::httproutes::HTTPRoute>>::new();
        let mut gateway_tcp_route_map =
            HashMap::<String, Vec<gateway_api::experimental::tcproutes::TCPRoute>>::new();
        let mut gateway_tls_route_map =
            HashMap::<String, Vec<gateway_api::experimental::tlsroutes::TLSRoute>>::new();
        for route in route_list.items {
            for pr in route.spec.parent_refs.clone().unwrap_or_default() {
                let parent_name = pr.name;
                gateway_router_map
                    .entry(parent_name)
                    .or_default()
                    .push(route.clone());
            }
        }
        for route in tcp_route_list.items {
            for pr in route.spec.parent_refs.clone().unwrap_or_default() {
                gateway_tcp_route_map
                    .entry(pr.name)
                    .or_default()
                    .push(route.clone());
            }
        }
        for route in tls_route_list.items {
            for pr in route.spec.parent_refs.clone().unwrap_or_default() {
                gateway_tls_route_map
                    .entry(pr.name)
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
                tcp_routes: BTreeMap::new(),
                tls_routes: BTreeMap::new(),
            };

            for route in gateway_router_map.remove(&gateway_name).unwrap_or_default() {
                let route_name = route.name_any();
                gateway_data.http_routes.insert(route_name, route);
            }
            for route in gateway_tcp_route_map.remove(&gateway_name).unwrap_or_default() {
                let route_name = route.name_any();
                gateway_data.tcp_routes.insert(route_name, route);
            }
            for route in gateway_tls_route_map.remove(&gateway_name).unwrap_or_default() {
                let route_name = route.name_any();
                gateway_data.tls_routes.insert(route_name, route);
            }
            gathered_gateways.gateways.insert(gateway_name, gateway_data);
        }
        Ok(gathered_gateways)
    }
}
