use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap, HashSet},
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use gateway_api::httproutes::{
    HTTPRouteRulesMatchesHeaders, HTTPRouteRulesMatchesHeadersType, HTTPRouteRulesMatchesMethod,
    HTTPRouteRulesMatchesPathType, HTTPRouteRulesMatchesQueryParams,
    HTTPRouteRulesMatchesQueryParamsType,
};
use http::HeaderName;
use kube::{ResourceExt, api::ListParams};
use serde::{Deserialize, Serialize};
use switchboard_custom_config::{
    ConfigWithFormat, K8sResource, Link, SerdeValue,
    switchboard_serde_value::{self, value},
};
use switchboard_http_router::{
    rule::{HeaderMatch, QueryMatch, RuleMatch},
    serde::rule::{
        HeaderMatchSerde, QueryMatchSerde, RegexOrExactSerde, RuleBucketSerde, RuleMatchSerde,
    },
};
use switchboard_model::{
    Config, Listener, TcpServiceConfig, Tls,
    services::{
        self,
        http::{
            InstanceData, InstanceId, NodeOutput, NodePort, NodeTarget, WithOutputs,
            consts::ROUTER_CLASS_ID,
        },
    },
    tcp_route::TcpRoute,
};

use crate::{ControllerContext, resolve::k8s::K8sResolver};

mod backend;
mod filter;
fn target_name(name: &str, namespace: Option<&str>, port: Option<u16>) -> String {
    match (namespace, port) {
        (Some(ns), Some(p)) => format!("{}.{}.port{}", name, ns, p),
        (Some(ns), None) => format!("{}.{}", name, ns),
        (None, Some(p)) => format!("{}.port{}", name, p),
        (None, None) => name.to_string(),
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct K8sGatewayResourceConfig {
    pub gateway_namespace: String,
}

impl Default for K8sGatewayResourceConfig {
    fn default() -> Self {
        Self {
            gateway_namespace: "default".to_string(),
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct K8sGatewayGatheredResource {
    pub gateway_classes: BTreeMap<String, K8sGateways>,
}

#[derive(Debug, Clone, Default)]
pub struct K8sGateways {
    pub gateways: BTreeMap<String, K8sGatewayGatewayData>,
}

pub fn k8s_method_to_http_method(method: &HTTPRouteRulesMatchesMethod) -> String {
    match method {
        HTTPRouteRulesMatchesMethod::Get => "GET",
        HTTPRouteRulesMatchesMethod::Head => "HEAD",
        HTTPRouteRulesMatchesMethod::Post => "POST",
        HTTPRouteRulesMatchesMethod::Put => "PUT",
        HTTPRouteRulesMatchesMethod::Delete => "DELETE",
        HTTPRouteRulesMatchesMethod::Connect => "CONNECT",
        HTTPRouteRulesMatchesMethod::Options => "OPTIONS",
        HTTPRouteRulesMatchesMethod::Trace => "TRACE",
        HTTPRouteRulesMatchesMethod::Patch => "PATCH",
    }
    .to_string()
}

pub fn k8s_header_match_to_http_header_match(
    header: &HTTPRouteRulesMatchesHeaders,
) -> HeaderMatchSerde {
    let header_name = header.name.clone();
    match header.r#type {
        Some(HTTPRouteRulesMatchesHeadersType::Exact) | None => HeaderMatchSerde {
            header_name,
            header_value: RegexOrExactSerde::Exact(header.value.clone()),
        },
        Some(HTTPRouteRulesMatchesHeadersType::RegularExpression) => HeaderMatchSerde {
            header_name,
            header_value: RegexOrExactSerde::Regex(header.value.clone()),
        },
    }
}

pub fn k8s_query_match_to_http_query_match(
    query: &HTTPRouteRulesMatchesQueryParams,
) -> QueryMatchSerde {
    match query.r#type {
        Some(HTTPRouteRulesMatchesQueryParamsType::Exact) | None => QueryMatchSerde {
            query_name: query.name.clone(),
            query_value: RegexOrExactSerde::Exact(query.value.clone()),
        },
        Some(HTTPRouteRulesMatchesQueryParamsType::RegularExpression) => QueryMatchSerde {
            query_name: query.name.clone(),
            query_value: RegexOrExactSerde::Regex(query.value.clone()),
        },
    }
}

pub struct ServiceBuilder {
    pub config: switchboard_model::Config<SerdeValue, K8sResource>,
    pub context: ControllerContext,
}
#[derive(thiserror::Error, Debug)]
pub enum ServiceBuilderError {
    #[error("HTTP Router build error: {0}")]
    HttpRouterBuildError(#[from] switchboard_http_router::error::BuildError),
    #[error("Serde value error: {0}")]
    SerdeValueError(#[from] switchboard_serde_value::Error),
    #[error("K8s resolve error: {0}")]
    K8sResolveError(#[from] crate::resolve::k8s::K8sResolveError),
}
impl ServiceBuilder {
    pub fn new(context: ControllerContext) -> Self {
        Self {
            config: switchboard_model::Config::<SerdeValue, K8sResource>::default(),
            context,
        }
    }
    pub async fn resolve(self) -> Result<switchboard_model::Config, ServiceBuilderError> {
        let switchboard_model::Config::<SerdeValue, K8sResource> {
            tcp_services,
            tcp_listeners,
            tcp_routes,
            tls,
        } = self.config;
        let mut resolved_tls = BTreeMap::new();
        let resolver = K8sResolver::new(self.context);
        for (name, tls_link) in tls {
            let Tls {
                resolver: resource,
                options,
            } = tls_link;
            let tls_param = resolver.fetch_tls_cert_params(&resource).await?;
            let resolved_tls_resolver = tls_param.into();
            resolved_tls.insert(
                name,
                Tls {
                    resolver: resolved_tls_resolver,
                    options,
                },
            );
        }
        let resolved_config = switchboard_model::Config {
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
        let http_gateway_config = HttpGatewayBuilder::new().build_router_from_k8s(gateway)?;
        let value = switchboard_serde_value::SerdeValue::serialize_from(&http_gateway_config)?;
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

pub struct HttpGatewayBuilder {
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

    pub fn build_router_from_k8s(
        mut self,
        gateway: &K8sGatewayGatewayData,
    ) -> Result<services::http::Config, switchboard_http_router::error::BuildError> {
        let mut outputs = <BTreeMap<NodePort, NodeOutput>>::new();
        let mut router = switchboard_http_router::serde::RouterSerde::<NodePort>::default();
        // let listener = gateway.gateway.spec.listeners;
        for (route_name, route) in &gateway.http_routes {
            let mut path_tree =
                switchboard_http_router::serde::path::PathTreeSerdeMapStyle::<NodePort>::default();
            #[derive(Debug, Clone, Hash, PartialEq, Eq)]
            pub enum BucketKey {
                Matchit(String),
                Regex(String),
            }

            if let Some(rules_list) = &route.spec.rules {
                for (rule_set_index, rules) in rules_list.iter().enumerate() {
                    let mut buckets = HashMap::<BucketKey, Vec<RuleMatchSerde>>::new();
                    let rule_name = rules
                        .name
                        .as_deref()
                        .map(Cow::Borrowed)
                        .unwrap_or(Cow::Owned(format!("rule-{}", rule_set_index)));
                    let target_name = format!("{}-{}", route_name, rule_name);
                    let route_out_port = NodePort::from(target_name.as_str());

                    // build output target
                    let output_target = if let Some(k8s_backend_refs) = rules.backend_refs.as_ref()
                    {
                        self.build_backend_instance_from_k8s_backend_ref(
                            &target_name,
                            k8s_backend_refs,
                        )
                    } else {
                        self.internal_error_500_page_instance_id.clone().into()
                    };
                    let mut node_output = NodeOutput {
                        filters: vec![],
                        target: output_target,
                    };
                    if let Some(filters) = &rules.filters {
                        for (index, filter) in filters.iter().enumerate() {
                            let filter_name = filter::filter_id(route_name, &rule_name, index);
                            let filter_instance =
                                filter::build_filter_instance_from_k8s_router_filter(&filter);
                            self.config
                                .flow
                                .instances
                                .insert(filter_name.clone(), filter_instance);
                            node_output.filters.push(filter_name.into());
                        }
                    }

                    // setup router rules
                    if let Some(matches) = &rules.matches {
                        for k8s_match in matches {
                            let path = k8s_match.path.as_ref().cloned().unwrap_or_default();
                            let match_type = path
                                .r#type
                                .unwrap_or(HTTPRouteRulesMatchesPathType::PathPrefix);
                            let route_path = path.value.as_deref().unwrap_or("/");
                            // build rule
                            let rule = {
                                let headers = k8s_match
                                    .headers
                                    .as_ref()
                                    .map(|hs| {
                                        hs.iter()
                                            .map(k8s_header_match_to_http_header_match)
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                let queries = k8s_match
                                    .query_params
                                    .as_ref()
                                    .map(|qs| {
                                        qs.iter().map(k8s_query_match_to_http_query_match).collect()
                                    })
                                    .unwrap_or_default();
                                let rule = RuleMatchSerde {
                                    method: k8s_match
                                        .method
                                        .as_ref()
                                        .map(k8s_method_to_http_method),
                                    headers,
                                    queries,
                                };
                                rule
                            };
                            let tree_key = match match_type {
                                HTTPRouteRulesMatchesPathType::Exact => {
                                    BucketKey::Matchit(route_path.to_string())
                                }
                                HTTPRouteRulesMatchesPathType::PathPrefix => BucketKey::Matchit(
                                    format!("{}{{*rest}}", route_path.trim_end_matches('/')),
                                ),
                                HTTPRouteRulesMatchesPathType::RegularExpression => {
                                    BucketKey::Regex(route_path.to_string())
                                }
                            };
                            buckets.entry(tree_key).or_default().push(rule);
                        }
                    }
                    for (key, bucket) in buckets {
                        let mut rule_bucket =
                            RuleBucketSerde::<NodePort>::new(route_out_port.clone());
                        rule_bucket.rules = bucket;
                        rule_bucket.sort();
                        match key {
                            BucketKey::Matchit(route_path) => {
                                path_tree.insert(route_path, rule_bucket.into());
                            }
                            BucketKey::Regex(regex_str) => {
                                path_tree.insert(regex_str, rule_bucket.into());
                            }
                        }
                    }
                    outputs.insert(route_out_port, node_output);
                }
            }
            if let Some(hostnames) = &route.spec.hostnames {
                for hostname in hostnames {
                    router.hostname.insert(hostname.clone(), path_tree.clone());
                }
            } else {
                router.hostname.insert("*".to_string(), path_tree);
            }
        }
        let config = value!({
            "outputs": outputs,
            "hostname": router.hostname,
        });
        let router_instance = InstanceData {
            class: services::http::ClassId::std(ROUTER_CLASS_ID),
            name: None,
            r#type: services::http::InstanceType::Node,
            config,
        };
        let instance_id = InstanceId::new(gateway.gateway.name_any());
        self.config
            .flow
            .instances
            .insert(instance_id, router_instance);
        Ok(self.config)
    }

    pub fn build(self) -> services::http::Config {
        self.config
    }
}

#[derive(Debug, Clone, Default)]
pub struct K8sGatewayGatewayData {
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
impl crate::ControllerContext {
    pub async fn build_config_from_k8s(
        &self,
    ) -> Result<switchboard_model::Config, K8sGatewayResourceError> {
        let gateways = self.gather_k8s_gateway_config().await?;
        let mut builder = ServiceBuilder::new(self.clone());
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
        let Some(k8s_gateway_resource_config) = self.controller_config.resource.gateway.k8s.clone()
        else {
            tracing::debug!("No K8s Gateway resource config found, skipping K8s Gateway gathering");
            return Ok(gathered_gateways);
        };
        let Some(client) = self.k8s_client.clone() else {
            return Err(K8sGatewayResourceError::NoK8sClient);
        };
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
            &k8s_gateway_resource_config.gateway_namespace,
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
