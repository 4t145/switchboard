use std::borrow::Cow;

use switchboard_model::services::http::NodePort;

use crate::resolve::k8s::service_config::{HttpBuildingRouter, HttpGatewayBuilder, route};

impl HttpGatewayBuilder {
    pub fn build_route(
        &mut self,
        route_name: &str,
        route: &gateway_api::httproutes::HTTPRoute,
        building_router: &mut HttpBuildingRouter,
    ) {
        let mut path_tree =
            switchboard_http_router::serde::path::PathTreeSerdeMapStyle::<NodePort>::default();

        if let Some(rules_list) = &route.spec.rules {
            for (rule_set_index, rules) in rules_list.iter().enumerate() {
                let rule_name = rules
                    .name
                    .as_deref()
                    .map(Cow::Borrowed)
                    .unwrap_or(Cow::Owned(format!("rule-{}", rule_set_index)));
                let target_name = format!("{}-{}", route_name, rule_name);
                let (route_out_port, node_output) =
                    self.build_rule(rules, &target_name, &mut path_tree);
                building_router.outputs.insert(route_out_port, node_output);
            }
        }
        if let Some(hostnames) = &route.spec.hostnames {
            for hostname in hostnames {
                if let Some(existed_path_tree) = building_router.router.hostname.get_mut(hostname) {
                    existed_path_tree.merge(path_tree.clone());
                } else {
                    building_router
                        .router
                        .hostname
                        .insert(hostname.clone(), path_tree.clone());
                }
            }
        } else {
            building_router
                .router
                .hostname
                .insert("*".to_string(), path_tree);
        }
    }
}
