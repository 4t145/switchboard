use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

use gateway_api::httproutes::{
    HTTPRouteRulesMatchesHeaders, HTTPRouteRulesMatchesHeadersType, HTTPRouteRulesMatchesMethod,
    HTTPRouteRulesMatchesPathType, HTTPRouteRulesMatchesQueryParams,
    HTTPRouteRulesMatchesQueryParamsType,
};
use switchboard_http_router::serde::{
    path::PathTreeSerdeMapStyle,
    rule::{
        HeaderMatchSerde, QueryMatchSerde, RegexOrExactSerde, RuleBucketSerde,
        RuleBucketSimplifiedSerde, RuleMatchSerde,
    },
};
use switchboard_model::services::http::{NodeOutput, NodePort};

use crate::resolve::k8s::service_config::filter;

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

impl super::HttpGatewayBuilder {
    pub fn build_rule(
        &mut self,
        rules: &gateway_api::httproutes::HTTPRouteRules,
        target_name: &str,
        path_tree: &mut PathTreeSerdeMapStyle<NodePort>,
    ) -> (NodePort, NodeOutput) {
        let mut buckets = HashMap::<String, Vec<RuleMatchSerde>>::new();

        let route_out_port = NodePort::from(target_name);

        // build output target
        let output_target = if let Some(k8s_backend_refs) = rules.backend_refs.as_ref() {
            self.build_backend_instance_from_k8s_backend_ref(&target_name, k8s_backend_refs)
        } else {
            self.internal_error_500_page_instance_id.clone().into()
        };
        let mut node_output = NodeOutput {
            filters: vec![],
            target: output_target,
        };
        if let Some(filters) = &rules.filters {
            for (index, filter) in filters.iter().enumerate() {
                let filter_name = filter::filter_id(target_name, index);
                let filter_instance = filter::build_filter_instance_from_k8s_router_filter(&filter);
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
                        .map(|qs| qs.iter().map(k8s_query_match_to_http_query_match).collect())
                        .unwrap_or_default();
                    let rule = RuleMatchSerde {
                        method: k8s_match.method.as_ref().map(k8s_method_to_http_method),
                        headers,
                        queries,
                    };
                    rule
                };
                let tree_key = match match_type {
                    HTTPRouteRulesMatchesPathType::Exact => route_path.to_string(),
                    HTTPRouteRulesMatchesPathType::PathPrefix => {
                        format!("{}{{*rest}}", route_path.trim_end_matches('/'))
                    }
                    HTTPRouteRulesMatchesPathType::RegularExpression => route_path.to_string(),
                };
                buckets.entry(tree_key).or_default().push(rule);
            }
        }
        for (key, bucket) in buckets {
            let mut rule_bucket = RuleBucketSerde::<NodePort>::new(route_out_port.clone());
            rule_bucket.rules = bucket;
            rule_bucket.sort();
            path_tree.insert(key, rule_bucket.into());
        }
        (route_out_port, node_output)
    }
}
