pub(super) fn parent_ref_matches_listener(
    parent_name: &str,
    parent_port: Option<i32>,
    parent_section_name: Option<&str>,
    gateway_name: &str,
    listener_name: &str,
    listener_port: u16,
) -> bool {
    if parent_name != gateway_name {
        return false;
    }
    if let Some(port) = parent_port
        && port != i32::from(listener_port)
    {
        return false;
    }
    if let Some(section_name) = parent_section_name
        && section_name != listener_name
    {
        return false;
    }
    true
}

pub(super) fn tcproute_attaches_listener(
    route: &gateway_api::experimental::tcproutes::TCPRoute,
    gateway_name: &str,
    listener_name: &str,
    listener_port: u16,
) -> bool {
    route
        .spec
        .parent_refs
        .as_ref()
        .map(|parent_refs| {
            parent_refs.iter().any(|pr| {
                parent_ref_matches_listener(
                    &pr.name,
                    pr.port,
                    pr.section_name.as_deref(),
                    gateway_name,
                    listener_name,
                    listener_port,
                )
            })
        })
        .unwrap_or(false)
}

pub(super) fn tlsroute_attaches_listener(
    route: &gateway_api::experimental::tlsroutes::TLSRoute,
    gateway_name: &str,
    listener_name: &str,
    listener_port: u16,
) -> bool {
    route
        .spec
        .parent_refs
        .as_ref()
        .map(|parent_refs| {
            parent_refs.iter().any(|pr| {
                parent_ref_matches_listener(
                    &pr.name,
                    pr.port,
                    pr.section_name.as_deref(),
                    gateway_name,
                    listener_name,
                    listener_port,
                )
            })
        })
        .unwrap_or(false)
}
