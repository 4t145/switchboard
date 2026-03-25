use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
};

use kube::ResourceExt;
use switchboard_model::{
    Listener, TcpServiceConfig, Tls, switchboard_serde_value, tcp_route::TcpRoute,
};
use switchboard_tcp::{
    TcpConfig, TlsStrategyConfig,
    balancer::BalancerStrategyConfig,
    outbound::{Outbound, OutboundMap, OutboundName},
};

use super::{K8sGatewayGatewayData, K8sResource, ServiceBuilder, ServiceBuilderError, target_name};

mod match_parent;
mod outbound;

impl ServiceBuilder {
    pub fn build_tcp_services(
        &mut self,
        gateway: &K8sGatewayGatewayData,
    ) -> Result<Vec<String>, ServiceBuilderError> {
        let gateway_name = gateway.gateway.name_any();
        let mut service_names = Vec::new();

        for listener in &gateway.gateway.spec.listeners {
            if listener.protocol != "TLS" {
                continue;
            }
            let port = listener.port as u16;
            let bind = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
            let listener_name = listener.name.clone();
            let mode = listener
                .tls
                .as_ref()
                .and_then(|tls| tls.mode.clone())
                .unwrap_or(gateway_api::gateways::GatewayListenersTlsMode::Terminate);

            match mode {
                gateway_api::gateways::GatewayListenersTlsMode::Passthrough => {
                    let mut sni_map = HashMap::<String, Outbound>::new();
                    for route in gateway.tls_routes.values() {
                        if !match_parent::tlsroute_attaches_listener(
                            route,
                            &gateway_name,
                            &listener_name,
                            port,
                        ) {
                            continue;
                        }
                        let Some(outbound) = outbound::build_tcp_outbound_from_tlsroute(route)
                        else {
                            continue;
                        };
                        for hostname in &route.spec.hostnames {
                            sni_map.insert(hostname.clone(), outbound.clone());
                        }
                    }

                    if sni_map.is_empty() {
                        continue;
                    }

                    let service_name = format!("tcp-gateway-{}-{}", gateway_name, listener_name);
                    let strategy_config = TlsStrategyConfig::Passthrough(sni_map);
                    let tcp_config = TcpConfig {
                        strategy_config,
                        balancer: BalancerStrategyConfig::RoundRobin,
                    };
                    let value = switchboard_serde_value::SerdeValue::serialize_from(&tcp_config)?;
                    self.config.tcp_services.insert(
                        service_name.clone(),
                        TcpServiceConfig {
                            provider: "tcp".to_string(),
                            name: service_name.clone(),
                            config: Some(value),
                            description: Some(format!(
                                "K8s TCP Gateway passthrough for {} listener {}",
                                gateway_name, listener_name
                            )),
                        },
                    );
                    self.config.tcp_listeners.insert(
                        bind,
                        Listener {
                            bind,
                            description: Some(format!(
                                "tls passthrough listener {} for {}",
                                listener_name, gateway_name
                            )),
                        },
                    );
                    self.config.tcp_routes.insert(
                        bind,
                        TcpRoute {
                            bind,
                            service: service_name.clone(),
                            tls: None,
                        },
                    );
                    service_names.push(service_name);
                }
                gateway_api::gateways::GatewayListenersTlsMode::Terminate => {
                    let mut merged = OutboundMap::default();
                    for route in gateway.tcp_routes.values() {
                        if !match_parent::tcproute_attaches_listener(
                            route,
                            &gateway_name,
                            &listener_name,
                            port,
                        ) {
                            continue;
                        }
                        let Some(outbound) = outbound::build_tcp_outbound_from_tcproute(route)
                        else {
                            continue;
                        };
                        match outbound {
                            Outbound::Single(endpoint) => {
                                merged.insert(
                                    OutboundName::new(target_name(
                                        &endpoint.host,
                                        None,
                                        Some(endpoint.port),
                                    )),
                                    endpoint,
                                );
                            }
                            Outbound::NamedMap(map) => {
                                merged.extend(map);
                            }
                        }
                    }
                    if merged.is_empty() {
                        continue;
                    }
                    let outbound = if merged.len() == 1 {
                        let endpoint = match merged.into_values().next() {
                            Some(endpoint) => endpoint,
                            None => continue,
                        };
                        Outbound::Single(endpoint)
                    } else {
                        Outbound::NamedMap(merged)
                    };

                    let Some(listener_tls) = listener.tls.as_ref() else {
                        continue;
                    };
                    let Some(listener_certificate_refs) = listener_tls.certificate_refs.as_ref()
                    else {
                        continue;
                    };

                    let service_name = format!("tcp-gateway-{}-{}", gateway_name, listener_name);
                    let strategy_config = TlsStrategyConfig::Terminate(outbound);
                    let tcp_config = TcpConfig {
                        strategy_config,
                        balancer: BalancerStrategyConfig::RoundRobin,
                    };
                    let value = switchboard_serde_value::SerdeValue::serialize_from(&tcp_config)?;

                    let mut tls_name = None;
                    if let Some(listener_certificate_ref) = listener_certificate_refs.first() {
                        let tls_resource = K8sResource::new(
                            listener_certificate_ref.namespace.clone(),
                            listener_certificate_ref.name.clone(),
                        );
                        let name = format!(
                            "k8s-gateway-{}-listener-{}-tls",
                            gateway_name, listener_name
                        );
                        self.config.tls.insert(
                            name.clone(),
                            Tls {
                                resolver: tls_resource,
                                options: Default::default(),
                            },
                        );
                        tls_name = Some(name);
                    }

                    self.config.tcp_services.insert(
                        service_name.clone(),
                        TcpServiceConfig {
                            provider: "tcp".to_string(),
                            name: service_name.clone(),
                            config: Some(value),
                            description: Some(format!(
                                "K8s TCP Gateway terminate for {} listener {}",
                                gateway_name, listener_name
                            )),
                        },
                    );
                    self.config.tcp_listeners.insert(
                        bind,
                        Listener {
                            bind,
                            description: Some(format!(
                                "tls terminate listener {} for {}",
                                listener_name, gateway_name
                            )),
                        },
                    );
                    self.config.tcp_routes.insert(
                        bind,
                        TcpRoute {
                            bind,
                            service: service_name.clone(),
                            tls: tls_name,
                        },
                    );
                    service_names.push(service_name);
                }
            }
        }

        Ok(service_names)
    }
}
