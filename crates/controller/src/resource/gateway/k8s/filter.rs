use gateway_api::apis::standard::httproutes::{
    HTTPRouteRulesFilters, HTTPRouteRulesFiltersExtensionRef,
    HTTPRouteRulesFiltersRequestHeaderModifier, HTTPRouteRulesFiltersRequestHeaderModifierAdd,
    HTTPRouteRulesFiltersRequestHeaderModifierSet, HTTPRouteRulesFiltersRequestMirror,
    HTTPRouteRulesFiltersRequestMirrorBackendRef, HTTPRouteRulesFiltersRequestMirrorFraction,
    HTTPRouteRulesFiltersRequestRedirect, HTTPRouteRulesFiltersRequestRedirectPath,
    HTTPRouteRulesFiltersRequestRedirectPathType, HTTPRouteRulesFiltersRequestRedirectScheme,
    HTTPRouteRulesFiltersRequestRedirectStatusCode, HTTPRouteRulesFiltersResponseHeaderModifier,
    HTTPRouteRulesFiltersResponseHeaderModifierAdd, HTTPRouteRulesFiltersResponseHeaderModifierSet,
    HTTPRouteRulesFiltersType, HTTPRouteRulesFiltersUrlRewrite,
    HTTPRouteRulesFiltersUrlRewritePath, HTTPRouteRulesFiltersUrlRewritePathType,
};
use switchboard_custom_config::switchboard_serde_value::{SerdeValue, value};
use switchboard_model::services::http::{
    ClassId, InstanceData, InstanceId, InstanceType,
    consts::{
        FILTER_REQUEST_HEADER_MODIFY_CLASS_ID, FILTER_REQUEST_MIRROR_CLASS_ID,
        FILTER_REQUEST_REDIRECT_CLASS_ID, FILTER_RESPONSE_HEADER_MODIFY_CLASS_ID,
        FILTER_URL_REWRITE_CLASS_ID,
    },
};
pub fn filter_id(route: &str, rule: &str, index: usize) -> InstanceId {
    InstanceId::new(format!("filter-{}-{}-{}", route, rule, index))
}
pub fn build_filter_instance_from_k8s_filter(k8s_filter: &HTTPRouteRulesFilters) -> InstanceData {
    match k8s_filter.r#type {
        HTTPRouteRulesFiltersType::RequestHeaderModifier => {
            if let Some(filter) = &k8s_filter.request_header_modifier {
                build_request_header_modify(filter)
            } else {
                build_request_header_modify(&Default::default())
            }
        }
        HTTPRouteRulesFiltersType::ResponseHeaderModifier => {
            if let Some(filter) = &k8s_filter.response_header_modifier {
                build_response_header_modify(filter)
            } else {
                build_response_header_modify(&Default::default())
            }
        }
        HTTPRouteRulesFiltersType::RequestMirror => {
            if let Some(filter) = &k8s_filter.request_mirror {
                build_request_mirror(filter)
            } else {
                build_request_mirror(&Default::default())
            }
        }
        HTTPRouteRulesFiltersType::RequestRedirect => {
            if let Some(filter) = &k8s_filter.request_redirect {
                build_redirect(filter)
            } else {
                build_redirect(&Default::default())
            }
        }
        HTTPRouteRulesFiltersType::UrlRewrite => {
            if let Some(filter) = &k8s_filter.url_rewrite {
                build_url_rewrite(filter)
            } else {
                build_url_rewrite(&Default::default())
            }
        }
        HTTPRouteRulesFiltersType::ExtensionRef => InstanceData {
            name: None,
            class: ClassId::std("extension-ref"),
            r#type: InstanceType::Filter,
            config: value!({}),
        },
    }
}

fn build_request_header_modify(
    filter: &HTTPRouteRulesFiltersRequestHeaderModifier,
) -> InstanceData {
    let class = ClassId::std(FILTER_REQUEST_HEADER_MODIFY_CLASS_ID);
    let extend_headers = filter
        .add
        .as_ref()
        .map(|add| {
            add.iter()
                .map(|header| (header.name.clone(), header.value.clone()))
                .collect::<Vec<(String, String)>>()
        })
        .unwrap_or_default();

    let set_headers = filter
        .set
        .as_ref()
        .map(|set| {
            set.iter()
                .map(|header| (header.name.clone(), header.value.clone()))
                .collect::<Vec<(String, String)>>()
        })
        .unwrap_or_default();

    let remove_headers = filter
        .remove
        .as_ref()
        .map(|remove| {
            remove
                .iter()
                .map(|header_name| header_name.clone())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let config = value!({
        "set": set_headers,
        "extend": extend_headers,
        "remove": remove_headers,
    });
    InstanceData {
        name: None,
        class,
        r#type: InstanceType::Filter,
        config,
    }
}

fn build_response_header_modify(
    filter: &HTTPRouteRulesFiltersResponseHeaderModifier,
) -> InstanceData {
    let class = ClassId::std(FILTER_RESPONSE_HEADER_MODIFY_CLASS_ID);
    let extend_headers = filter
        .add
        .as_ref()
        .map(|add| {
            add.iter()
                .map(|header| (header.name.clone(), header.value.clone()))
                .collect::<Vec<(String, String)>>()
        })
        .unwrap_or_default();

    let set_headers = filter
        .set
        .as_ref()
        .map(|set| {
            set.iter()
                .map(|header| (header.name.clone(), header.value.clone()))
                .collect::<Vec<(String, String)>>()
        })
        .unwrap_or_default();

    let remove_headers = filter
        .remove
        .as_ref()
        .map(|remove| {
            remove
                .iter()
                .map(|header_name| header_name.clone())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let config = value!({
        "set": set_headers,
        "extend": extend_headers,
        "remove": remove_headers,
    });
    InstanceData {
        name: None,
        class,
        r#type: InstanceType::Filter,
        config,
    }
}

fn build_request_mirror(filter: &HTTPRouteRulesFiltersRequestMirror) -> InstanceData {
    let class = ClassId::std(FILTER_REQUEST_MIRROR_CLASS_ID);
    let HTTPRouteRulesFiltersRequestMirrorBackendRef {
        name,
        namespace,
        port,
        ..
    } = &filter.backend_ref;
    let target_name =
        super::target_name(name, namespace.as_deref(), port.as_ref().map(|p| *p as u16));
    let fraction = &filter.fraction;
    let possibility = if let Some(fraction) = fraction {
        let denominator = fraction.denominator.unwrap_or(100);
        let numerator = fraction.numerator.min(denominator);
        Some(format!("{}/{}", numerator, denominator))
    } else if let Some(percentage) = &filter.percent {
        let percentage = (*percentage).min(100);
        Some(format!("{}%", percentage))
    } else {
        None
    };
    let config = value!({
        "target": target_name,
        "possibility": possibility,
    });
    InstanceData {
        name: None,
        class,
        r#type: InstanceType::Filter,
        config,
    }
}

pub fn build_url_rewrite(filter: &HTTPRouteRulesFiltersUrlRewrite) -> InstanceData {
    let class = ClassId::std(FILTER_URL_REWRITE_CLASS_ID);
    let path = if let Some(path) = &filter.path {
        let path_value = match &path.r#type {
            HTTPRouteRulesFiltersUrlRewritePathType::ReplacePrefixMatch => {
                let value = path.replace_prefix_match.as_deref().unwrap_or("");
                format!("{}/{{*rest}}", value)
            }
            HTTPRouteRulesFiltersUrlRewritePathType::ReplaceFullPath => {
                path.replace_full_path.as_deref().unwrap_or("/").to_string()
            }
        };
        Some(path_value)
    } else {
        None
    };
    let hostname = filter.hostname.clone();
    let config = value!({
        "path": path,
        "hostname": hostname,
    });
    InstanceData {
        name: None,
        class,
        r#type: InstanceType::Filter,
        config,
    }
}

fn build_redirect(filter: &HTTPRouteRulesFiltersRequestRedirect) -> InstanceData {
    let class = ClassId::std(FILTER_REQUEST_REDIRECT_CLASS_ID);
    let status_code = filter.status_code.map(|code| code as u16);
    let scheme = filter
        .scheme
        .clone()
        .map(|s| match s {
            HTTPRouteRulesFiltersRequestRedirectScheme::Http => "http",
            HTTPRouteRulesFiltersRequestRedirectScheme::Https => "https",
        })
        .clone();
    let path = if let Some(path) = &filter.path {
        let path_value = match &path.r#type {
            HTTPRouteRulesFiltersRequestRedirectPathType::ReplacePrefixMatch => {
                let value = path.replace_prefix_match.as_deref().unwrap_or("");
                format!("{}/{{*rest}}", value)
            }
            HTTPRouteRulesFiltersRequestRedirectPathType::ReplaceFullPath => {
                path.replace_full_path.as_deref().unwrap_or("/").to_string()
            }
        };
        Some(path_value)
    } else {
        None
    };
    let path = path.unwrap_or_default();
    let hostname = filter.hostname.clone();
    let to = if let Some(hostname) = &hostname {
        if let Some(scheme) = &scheme {
            format!("{}://{}{}", scheme, hostname, path)
        } else {
            format!("//{}{}", hostname, path)
        }
    } else {
        String::new()
    };
    let config = value!({
        "status_code": status_code,
        "to": to,
    });
    InstanceData {
        name: None,
        class,
        r#type: InstanceType::Filter,
        config,
    }
}
