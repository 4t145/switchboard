use gateway_api::{apis::standard::httproutes::{
    HTTPRouteRulesBackendRefsFilters, HTTPRouteRulesBackendRefsFiltersRequestMirrorBackendRef, HTTPRouteRulesBackendRefsFiltersType, HTTPRouteRulesBackendRefsFiltersUrlRewritePathType, HTTPRouteRulesFilters, HTTPRouteRulesFiltersExtensionRef, HTTPRouteRulesFiltersRequestHeaderModifier, HTTPRouteRulesFiltersRequestHeaderModifierAdd, HTTPRouteRulesFiltersRequestHeaderModifierSet, HTTPRouteRulesFiltersRequestMirror, HTTPRouteRulesFiltersRequestMirrorBackendRef, HTTPRouteRulesFiltersRequestMirrorFraction, HTTPRouteRulesFiltersRequestRedirect, HTTPRouteRulesFiltersRequestRedirectPath, HTTPRouteRulesFiltersRequestRedirectPathType, HTTPRouteRulesFiltersRequestRedirectScheme, HTTPRouteRulesFiltersRequestRedirectStatusCode, HTTPRouteRulesFiltersResponseHeaderModifier, HTTPRouteRulesFiltersResponseHeaderModifierAdd, HTTPRouteRulesFiltersResponseHeaderModifierSet, HTTPRouteRulesFiltersType, HTTPRouteRulesFiltersUrlRewrite, HTTPRouteRulesFiltersUrlRewritePath, HTTPRouteRulesFiltersUrlRewritePathType
}, httproutes::{HTTPRouteRulesBackendRefsFiltersRequestRedirectPathType, HTTPRouteRulesBackendRefsFiltersRequestRedirectScheme}};
use switchboard_custom_config::switchboard_serde_value::{SerdeValue, value};
use switchboard_model::services::http::{
    ClassId, InstanceData, InstanceId, InstanceType,
    consts::{
        FILTER_REQUEST_HEADER_MODIFY_CLASS_ID, FILTER_REQUEST_MIRROR_CLASS_ID,
        FILTER_REQUEST_REDIRECT_CLASS_ID, FILTER_RESPONSE_HEADER_MODIFY_CLASS_ID,
        FILTER_URL_REWRITE_CLASS_ID,
    },
};
pub fn filter_id(target: &str, index: usize) -> InstanceId {
    InstanceId::new(format!("filter-{}-{}", target, index))
}
use std::borrow::Cow;

macro_rules! derive_filter_from {
    (
        $fname: ident, $Filters: ident, $FiltersType: ident,
        $MirrorBackendRef: ident,
        $RewritePathType: ident,
        $RedirectSchema: ident,
        $RedirectPathType: ident,
    ) => {
        pub fn $fname(k8s_filter: &$Filters) -> InstanceData {
            match k8s_filter.r#type {
                $FiltersType::RequestHeaderModifier => {
                    let filter = k8s_filter
                        .request_header_modifier
                        .as_ref()
                        .map(|f| Cow::Borrowed(f))
                        .unwrap_or_else(|| Cow::Owned(Default::default()));
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
                $FiltersType::ResponseHeaderModifier => {
                    let filter = k8s_filter
                        .response_header_modifier
                        .as_ref()
                        .map(|f| Cow::Borrowed(f))
                        .unwrap_or_else(|| Cow::Owned(Default::default()));
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
                $FiltersType::RequestMirror => {
                    let filter = k8s_filter
                        .request_mirror
                        .as_ref()
                        .map(|f| Cow::Borrowed(f))
                        .unwrap_or_else(|| Cow::Owned(Default::default()));
                    let class = ClassId::std(FILTER_REQUEST_MIRROR_CLASS_ID);
                    let $MirrorBackendRef {
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
                $FiltersType::RequestRedirect => {
                    let filter = k8s_filter
                        .request_redirect
                        .as_ref()
                        .map(|f| Cow::Borrowed(f))
                        .unwrap_or_else(|| Cow::Owned(Default::default()));
                        let class = ClassId::std(FILTER_REQUEST_REDIRECT_CLASS_ID);
                    let status_code = filter.status_code.map(|code| code as u16);
                    let scheme = filter
                        .scheme
                        .clone()
                        .map(|s| match s {
                            $RedirectSchema::Http => "http",
                            $RedirectSchema::Https => "https",
                        })
                        .clone();
                    let path = if let Some(path) = &filter.path {
                        let path_value = match &path.r#type {
                            $RedirectPathType::ReplacePrefixMatch => {
                                let value = path.replace_prefix_match.as_deref().unwrap_or("");
                                format!("{}/{{*rest}}", value)
                            }
                            $RedirectPathType::ReplaceFullPath => {
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
                $FiltersType::UrlRewrite => {
                    let filter = k8s_filter
                        .url_rewrite
                        .as_ref()
                        .map(|f| Cow::Borrowed(f))
                        .unwrap_or_else(|| Cow::Owned(Default::default()));
                        let class = ClassId::std(FILTER_URL_REWRITE_CLASS_ID);
                    let path = if let Some(path) = &filter.path {
                        let path_value = match &path.r#type {
                            $RewritePathType::ReplacePrefixMatch => {
                                let value = path.replace_prefix_match.as_deref().unwrap_or("");
                                format!("{}/{{*rest}}", value)
                            }
                            $RewritePathType::ReplaceFullPath => {
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
                $FiltersType::ExtensionRef => InstanceData {
                    name: None,
                    class: ClassId::std("extension-ref"),
                    r#type: InstanceType::Filter,
                    config: value!({}),
                },
            }
        }
    };
}

derive_filter_from! {
    build_filter_instance_from_k8s_router_filter,
    HTTPRouteRulesFilters,
    HTTPRouteRulesFiltersType,
    HTTPRouteRulesFiltersRequestMirrorBackendRef,
    HTTPRouteRulesFiltersUrlRewritePathType,
    HTTPRouteRulesFiltersRequestRedirectScheme,
    HTTPRouteRulesFiltersRequestRedirectPathType,
}

derive_filter_from! {
    build_filter_instance_from_k8s_backend_filter,
    HTTPRouteRulesBackendRefsFilters,
    HTTPRouteRulesBackendRefsFiltersType,
    HTTPRouteRulesBackendRefsFiltersRequestMirrorBackendRef,
    HTTPRouteRulesBackendRefsFiltersUrlRewritePathType,
    HTTPRouteRulesBackendRefsFiltersRequestRedirectScheme,
    HTTPRouteRulesBackendRefsFiltersRequestRedirectPathType,
}
