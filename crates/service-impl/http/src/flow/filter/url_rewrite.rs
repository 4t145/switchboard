use std::{collections::HashMap, convert::Infallible, str::FromStr};

use http::{HeaderValue, StatusCode};

use serde::{Deserialize, Serialize};
use switchboard_http_router::utils::str_template::StrTemplate;
use switchboard_model::services::http::ClassId;

use crate::{
    DynRequest, DynResponse, consts::ERR_FILTER_URL_REWRITE,
    flow::filter::{FilterClass, FilterLike},
    utils::error_response,
};

#[derive(Clone, Deserialize, Serialize, bincode::Encode, bincode::Decode)]
pub struct UrlRewriteFilterConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UrlRewriteFilter {
    pub path: Option<StrTemplate>,
    pub hostname: Option<HeaderValue>,
}

impl UrlRewriteFilter {
    pub async fn rewrite(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &'_ mut crate::flow::FlowContext,
        next: super::Next,
    ) -> Result<DynResponse, UrlRewriteFilterError> {
        let (mut parts, body) = req.into_parts();
        if self.hostname.is_some() || self.path.is_some() {
            let mut uri_parts = parts.uri.into_parts();
            if let Some(path_template) = &self.path {
                let new_path = if path_template.is_literal() {
                    path_template.render(&HashMap::new())
                } else {
                    let mut vars = HashMap::new();
                    let captures = parts
                        .extensions
                        .get::<crate::extension::captures::Captures>();
                    if let Some(captures) = captures {
                        for (k, v) in &captures.captures {
                            vars.insert(k.as_ref(), v.as_ref());
                        }
                    }
                    if let Some(matched) = parts
                        .extensions
                        .get::<switchboard_http_router::RouterMatched<()>>()
                    {
                        for (k, v) in matched.path_tree_matched.captures_iter() {
                            vars.insert(k, v);
                        }
                    }
                    path_template.render(&vars)
                };
                if let Some(original_pq) = uri_parts.path_and_query {
                    let new_pq = if let Some((_, rest)) = original_pq.as_str().split_once('?') {
                        format!("{}?{}", new_path, rest)
                    } else {
                        new_path
                    };
                    uri_parts.path_and_query =
                        Some(http::uri::PathAndQuery::from_maybe_shared(new_pq)?);
                } else {
                    uri_parts.path_and_query =
                        Some(http::uri::PathAndQuery::from_maybe_shared(new_path)?);
                }
            }
            // rewrite hostname
            if let Some(host_header_value) = &self.hostname {
                uri_parts.authority = Some(host_header_value.as_bytes().try_into()?);
                parts
                    .headers
                    .insert(http::header::HOST, host_header_value.clone());
            }
            parts.uri = http::Uri::from_parts(uri_parts)?;
        }

        let req = DynRequest::from_parts(parts, body);
        Ok(next.call(req, ctx).await)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UrlRewriteFilterError {
    #[error("Invalid uri: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Invalid uri parts: {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
}

#[derive(Debug, thiserror::Error)]
pub enum UrlRewriteFilterConfigError {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    #[error("Invalid uri: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Invalid uri parts: {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),

    #[error("Invalid path template: {0}")]
    InvalidPathTemplate(#[from] Infallible),
}

impl FilterLike for UrlRewriteFilter {
    async fn call(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &mut crate::flow::FlowContext,
        next: super::Next,
    ) -> DynResponse {
        self.rewrite(req, ctx, next).await.unwrap_or_else(|e| {
            tracing::error!("Failed to rewrite request: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e, ERR_FILTER_URL_REWRITE)
        })
    }
}

pub struct UrlRewriteFilterClass;

impl FilterClass for UrlRewriteFilterClass {
    type Filter = UrlRewriteFilter;
    type Error = UrlRewriteFilterConfigError;
    type Config = UrlRewriteFilterConfig;

    fn id(&self) -> ClassId {
        ClassId::std("url-rewrite")
    }

    fn construct(&self, config: Self::Config) -> Result<Self::Filter, Self::Error> {
        let path = if let Some(path_str) = config.path {
            Some(StrTemplate::from_str(&path_str)?)
        } else {
            None
        };
        let hostname = if let Some(hostname_str) = config.hostname {
            Some(HeaderValue::from_str(&hostname_str)?)
        } else {
            None
        };
        Ok(UrlRewriteFilter { path, hostname })
    }
}
