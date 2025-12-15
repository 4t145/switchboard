use std::{
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
use switchboard_http_router::{
    rule::{HeaderMatch, QueryMatch, RuleMatch},
    serde::rule::{
        HeaderMatchSerde, QueryMatchSerde, RegexOrExactSerde, RuleBucketSerde, RuleMatchSerde,
    },
};

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

type RouteKey = Arc<str>;
pub fn build_router_from_k8s(
    gateway: &K8sGatewayGatewayData,
) -> Result<
    switchboard_http_router::serde::RouterSerde<RouteKey>,
    switchboard_http_router::error::BuildError,
> {
    // gateway.gateway.spec.listeners.as_ref();
    // build router
    let mut router = switchboard_http_router::serde::RouterSerde::<RouteKey>::default();
    for (route_name, route) in &gateway.http_routes {
        let mut path_tree =
            switchboard_http_router::serde::path::PathTreeSerde::<RouteKey>::default();
        let route_target = Arc::<str>::from(route_name.as_str());
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        pub enum BucketKey {
            Matchit(String),
            Regex(String),
        }
        pub struct BucketItem {
            rule: Option<RuleMatchSerde>,
            target: RouteKey,
        }
        let mut buckets = HashMap::<BucketKey, Vec<BucketItem>>::new();
        if let Some(rules_list) = &route.spec.rules {
            for rules in rules_list {
                if let Some(matches) = &rules.matches {
                    for k8s_match in matches {
                        let path = k8s_match.path.as_ref().cloned().unwrap_or_default();
                        let match_type = path
                            .r#type
                            .unwrap_or(HTTPRouteRulesMatchesPathType::PathPrefix);
                        let route_path = path.value.as_deref().unwrap_or("/");
                        // build rule
                        let rule = if k8s_match.headers.as_ref().is_none_or(Vec::is_empty)
                            && k8s_match.query_params.as_ref().is_none_or(Vec::is_empty)
                            && k8s_match.method.is_none()
                        {
                            None
                        } else {
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
                                method: k8s_match.method.as_ref().map(k8s_method_to_http_method),
                                headers,
                                queries,
                            };
                            Some(rule)
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
                        buckets.entry(tree_key).or_default().push(BucketItem {
                            rule,
                            target: route_target.clone(),
                        });
                    }
                }
            }
        }
        for (tree_key, tree_bucket) in buckets {
            let mut rule_bucket = RuleBucketSerde::<RouteKey>::new();
            for item in tree_bucket {
                if let Some(rule) = item.rule {
                    rule_bucket.add_rule(rule, item.target);
                } else {
                    rule_bucket.add_rule(RuleMatchSerde::fallback_rule(), item.target);
                }
            }
            match tree_key {
                BucketKey::Matchit(route_path) => {
                    path_tree.add_matchit_route(route_path, rule_bucket);
                }
                BucketKey::Regex(regex_str) => {
                    path_tree.add_regex_route(regex_str, rule_bucket);
                }
            }
        }
        if let Some(hostnames) = &route.spec.hostnames {
            for hostname in hostnames {
                router
                    .hostname_tree
                    .insert(hostname.clone(), path_tree.clone());
            }
        } else {
            router.hostname_tree.insert("*".to_string(), path_tree);
        }
    }   
    Ok(router)
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
        let mut flow_config = FlowConfig {
            entrypoint: NodeTarget {
                id: default_router_id.clone(),
                port: NodePort::Default,
            },
            instances: Default::default(),
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
        let Some(k8s_gateway_resource_config) =
            self.controller_config.resource_config.gateway.k8s.clone()
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
