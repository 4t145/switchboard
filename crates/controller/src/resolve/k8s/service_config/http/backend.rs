use std::collections::BTreeMap;

use gateway_api::httproutes::HTTPRouteRulesBackendRefs;
use switchboard_model::services::http::{
    consts::{BALANCER_CLASS_ID, REVERSE_PROXY_CLASS_ID},
    ClassId, InstanceData, InstanceId, InstanceType, NodeOutput, NodePort, NodeTarget,
};
use switchboard_model::switchboard_serde_value::value;

impl super::HttpGatewayBuilder {
    pub(crate) fn build_backend_instance_from_k8s_backend_ref(
        &mut self,
        target_name: &str,
        backend_refs: &[HTTPRouteRulesBackendRefs],
    ) -> NodeTarget {
        const DEFAULT_BALANCER_STRATEGY: &str = "RoundRobin";
        let mut balancer_outputs: BTreeMap<NodePort, NodeOutput> = BTreeMap::new();
        let balancer_strategy = DEFAULT_BALANCER_STRATEGY;
        let mut balancer_weights: BTreeMap<NodePort, u32> = BTreeMap::new();
        for (index, backend_ref) in backend_refs.iter().enumerate() {
            let port = backend_ref.port.map(|p| p as u16);
            let backend_service_host = if let Some(ns) = &backend_ref.namespace {
                format!("{}.{}", backend_ref.name, ns,)
            } else {
                backend_ref.name.clone()
            };
            let node_port = NodePort::from(format!("backend-{}-{}", index, backend_service_host));
            let mut filters_references = vec![];
            if let Some(filters) = &backend_ref.filters {
                for (filter_index, filter) in filters.iter().enumerate() {
                    let filter_unique_id = format!(
                        "{target_name}-backend-{}-{}-filter-{}",
                        index, backend_service_host, filter_index
                    );
                    let filter_instance_id = InstanceId::new(filter_unique_id);
                    filters_references.push(filter_instance_id.clone().into());
                    let filter =
                        super::filter::build_filter_instance_from_k8s_backend_filter(filter);
                    self.config
                        .flow
                        .filters
                        .insert(filter_instance_id, filter.without_type());
                }
            }
            let backend_service_instance_id: InstanceId =
                InstanceId::new(format!("{target_name}-{node_port}",));

            let output = NodeOutput {
                target: NodeTarget::from(backend_service_instance_id.clone()),
                filters: filters_references,
            };
            let weight = backend_ref.weight.unwrap_or(1) as u32;
            balancer_outputs.insert(node_port.clone(), output);
            balancer_weights.insert(node_port, weight);

            let backend = if let Some(port) = port {
                format!("{}:{}", backend_service_host, port)
            } else {
                backend_service_host
            };
            let reverse_proxy_service_instance = InstanceData {
                class: ClassId::std(REVERSE_PROXY_CLASS_ID),
                config: value!({
                    "backend": backend,
                    "scheme": "k8s",
                }),
                name: None,
                r#type: InstanceType::Node,
            };
            self.config.flow.filters.insert(
                backend_service_instance_id,
                reverse_proxy_service_instance.without_type(),
            );
        }
        let balancer_instance = if balancer_outputs.len() > 1 {
            let balancer_instance_id = InstanceId::new(format!("{target_name}-balancer"));
            let balancer_instance = InstanceData {
                class: ClassId::std(BALANCER_CLASS_ID),
                config: value!({
                    "type": balancer_strategy,
                    "config": balancer_weights,
                }),
                name: None,
                r#type: InstanceType::Node,
            };
            self.config.flow.filters.insert(
                balancer_instance_id.clone(),
                balancer_instance.without_type(),
            );
            balancer_instance_id
        } else if let Some(only_output) = balancer_outputs.into_values().next() {
            only_output.target.id.clone()
        } else {
            self.internal_error_500_page_instance_id.clone()
        };

        NodeTarget {
            id: balancer_instance,
            port: NodePort::Default,
        }
    }
}
