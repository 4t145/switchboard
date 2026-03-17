use std::{
    collections::BTreeMap,
    net::{Ipv4Addr, SocketAddr},
};

use kube::ResourceExt;
use switchboard_model::{
    Listener, TcpServiceConfig, Tls,
    services::{
        self,
        http::{
            InstanceData, InstanceId, NodeOutput, NodePort, NodeTarget, consts::ROUTER_CLASS_ID,
        },
    },
    switchboard_serde_value::{self, value},
    tcp_route::TcpRoute,
};

use super::{K8sGatewayGatewayData, K8sResource, ServiceBuilder, ServiceBuilderError};

mod backend;
mod filter;
mod route;
mod rule_match;

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
            .nodes
            .insert(router_id, router_instance.without_type());
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

impl ServiceBuilder {
    pub fn build_http_service(
        &mut self,
        gateway: &K8sGatewayGatewayData,
    ) -> Result<Option<String>, ServiceBuilderError> {
        let gateway_name = gateway.gateway.name_any();
        let listeners = gateway
            .gateway
            .spec
            .listeners
            .iter()
            .filter(|l| l.protocol == "HTTP" || l.protocol == "HTTPS")
            .collect::<Vec<_>>();
        if listeners.is_empty() {
            return Ok(None);
        }
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
            description: Some(format!("HTTP Gateway for K8s Gateway {}", gateway_name)),
        };
        for k8s_listener in listeners {
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
        Ok(Some(service_name))
    }
}
