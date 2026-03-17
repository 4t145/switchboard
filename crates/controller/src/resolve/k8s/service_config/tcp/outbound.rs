use switchboard_tcp::outbound::{Outbound, OutboundEndpoint, OutboundMap, OutboundName};

use super::super::target_name;

pub(super) fn backend_to_endpoint(
    name: &str,
    namespace: Option<&str>,
    port: Option<i32>,
    weight: Option<i32>,
) -> Option<(OutboundName, OutboundEndpoint)> {
    let port = u16::try_from(port?).ok()?;
    let host = if let Some(ns) = namespace {
        format!("{name}.{ns}")
    } else {
        name.to_string()
    };
    let endpoint = OutboundEndpoint {
        host,
        port,
        weight: weight.and_then(|w| u32::try_from(w).ok()),
    };
    let endpoint_name = OutboundName::new(target_name(name, namespace, Some(port)));
    Some((endpoint_name, endpoint))
}

pub(super) fn build_tcp_outbound_from_tcproute(
    route: &gateway_api::experimental::tcproutes::TCPRoute,
) -> Option<Outbound> {
    let mut outbounds = OutboundMap::default();
    for rule in &route.spec.rules {
        for backend_ref in &rule.backend_refs {
            if let Some((endpoint_name, endpoint)) = backend_to_endpoint(
                &backend_ref.name,
                backend_ref.namespace.as_deref(),
                backend_ref.port,
                backend_ref.weight,
            ) {
                outbounds.insert(endpoint_name, endpoint);
            }
        }
    }
    if outbounds.is_empty() {
        return None;
    }
    if outbounds.len() == 1 {
        let endpoint = outbounds.into_values().next()?;
        Some(Outbound::Single(endpoint))
    } else {
        Some(Outbound::NamedMap(outbounds))
    }
}

pub(super) fn build_tcp_outbound_from_tlsroute(
    route: &gateway_api::experimental::tlsroutes::TLSRoute,
) -> Option<Outbound> {
    let mut outbounds = OutboundMap::default();
    for rule in &route.spec.rules {
        for backend_ref in &rule.backend_refs {
            if let Some((endpoint_name, endpoint)) = backend_to_endpoint(
                &backend_ref.name,
                backend_ref.namespace.as_deref(),
                backend_ref.port,
                backend_ref.weight,
            ) {
                outbounds.insert(endpoint_name, endpoint);
            }
        }
    }
    if outbounds.is_empty() {
        return None;
    }
    if outbounds.len() == 1 {
        let endpoint = outbounds.into_values().next()?;
        Some(Outbound::Single(endpoint))
    } else {
        Some(Outbound::NamedMap(outbounds))
    }
}
