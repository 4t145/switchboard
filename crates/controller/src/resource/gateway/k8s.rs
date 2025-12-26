use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
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
    ConfigWithFormat,
    switchboard_serde_value::{self, value},
};
use switchboard_http_router::{
    rule::{HeaderMatch, QueryMatch, RuleMatch},
    serde::rule::{
        HeaderMatchSerde, QueryMatchSerde, RegexOrExactSerde, RuleBucketSerde, RuleMatchSerde,
    },
};
use switchboard_model::{
    Config, TcpServiceConfig,
    services::{
        self,
        http::{
            InstanceData, InstanceId, NodeOutput, NodePort, NodeTarget, WithOutputs,
            consts::ROUTER_CLASS_ID,
        },
    },
};

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
    pub gateway_classes: BTreeMap<String, K8sGatewayGatewayClassesData>,
}

#[derive(Debug, Clone, Default)]
pub struct K8sGatewayGatewayClassesData {
    pub gateway_class: gateway_api::gatewayclasses::GatewayClass,
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
    pub config: switchboard_model::Config,
}
#[derive(thiserror::Error, Debug)]
pub enum ServiceBuilderError {
    #[error("HTTP Router build error: {0}")]
    HttpRouterBuildError(#[from] switchboard_http_router::error::BuildError),
    #[error("Serde value error: {0}")]
    SerdeValueError(#[from] switchboard_serde_value::Error),
}
impl ServiceBuilder {
    pub fn new() -> Self {
        Self {
            config: switchboard_model::Config::default(),
        }
    }
    pub fn build_http_service(
        &mut self,
        gateway: &K8sGatewayGatewayData,
    ) -> Result<String, ServiceBuilderError> {
        let mut http_gateway_builder = HttpGatewayBuilder::new().build_router_from_k8s(gateway)?;
        let value = switchboard_serde_value::SerdeValue::serialize_from(&http_gateway_builder)?;
        let service_id = format!("http-gateway-{}", gateway.gateway.name_any());
        let service_instance = TcpServiceConfig {
            provider: "http".to_string(),
            name: service_id.clone(),
            config: Some(value),
            description: Some(format!(
                "HTTP Gateway for K8s Gateway {}",
                gateway.gateway.name_any()
            )),
        };
        for listener in &gateway.gateway.spec.listeners {
            let port = listener.port as u16;
            if let Some(tls) = &listener.tls {
                let mode = tls.certificate_refs.as_ref();
                
            }
        }
        todo!()
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

impl K8sGatewayGatewayData {
    pub fn build_http_config(&self) -> switchboard_model::services::http::Config {
        use switchboard_model::services::http::*;
        let default_router_id = InstanceId::new(self.gateway.name_any());
        // let router_instances;
        for (router_name, router) in &self.http_routes {
            if let Some(rules) = &router.spec.rules {
                for rule in rules {}
            }
        }
        let mut flow_config = FlowConfig::<ConfigWithFormat> {
            entrypoint: NodeTarget {
                id: default_router_id.clone(),
                port: NodePort::Default,
            },
            instances: Default::default(),
            nodes: Default::default(),
            filters: Default::default(),
            options: FlowOptions::default(),
        };

        todo!()
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
}

pub const GATEWAY_CONTROLLER_NAME: &str = "switchboard.io/gateway-controller";
impl crate::ControllerContext {
    pub async fn gather_k8s_gateway_config(
        &self,
    ) -> Result<K8sGatewayGatheredResource, K8sGatewayResourceError> {
        let mut gathered_resource = K8sGatewayGatheredResource::default();
        let Some(k8s_gateway_resource_config) = self.controller_config.resource.gateway.k8s.clone()
        else {
            return Ok(gathered_resource);
        };
        let Some(client) = self.k8s_client.clone() else {
            return Err(K8sGatewayResourceError::NoK8sClient);
        };
        let gateway_class_api =
            kube::Api::<gateway_api::gatewayclasses::GatewayClass>::all(client.clone());

        let gateway_list = gateway_class_api
            .list(&ListParams {
                field_selector: Some(format!("spec.controllerName={}", GATEWAY_CONTROLLER_NAME)),
                ..Default::default()
            })
            .await?;
        let gateway_class_list = gateway_list.items;

        // scan all gateway
        let gateway_api = kube::Api::<gateway_api::gateways::Gateway>::all(client.clone());
        for gateway_class in gateway_class_list {
            let gateway_class_name = gateway_class.name_any();
            let gateway_class_data = K8sGatewayGatewayClassesData {
                gateway_class,
                gateways: BTreeMap::new(),
            };
            let gateway_list = gateway_api
                .list(&ListParams {
                    field_selector: Some(format!("spec.gatewayClassName={}", gateway_class_name)),
                    ..Default::default()
                })
                .await?;
            for gateway in gateway_list.items {
                let gateway_name = gateway.name_any();
                let mut gateway_data = K8sGatewayGatewayData {
                    gateway: gateway.clone(),
                    http_routes: BTreeMap::new(),
                };
                let route_api = kube::Api::<gateway_api::httproutes::HTTPRoute>::namespaced(
                    client.clone(),
                    &k8s_gateway_resource_config.gateway_namespace,
                );
                let route_list = route_api
                    .list(&ListParams {
                        field_selector: Some(format!("spec.parentRefs.name={}", gateway_name)),
                        ..Default::default()
                    })
                    .await?;
                for route in route_list.items {
                    let route_name = route.name_any();
                    gateway_data.http_routes.insert(route_name, route.clone());
                }
            }
            gathered_resource
                .gateway_classes
                .insert(gateway_class_name, gateway_class_data);
        }
        Ok(gathered_resource)
    }
}
