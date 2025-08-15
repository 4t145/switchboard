use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    DynRequest, DynResponse, ERR_FILTER_REWRITE, flow::filter::FilterType, utils::error_response,
};

#[derive(Clone, Deserialize, Serialize, JsonSchema)]
pub struct RewriteLayer {
    pub host: Option<String>,
    pub schema: Option<String>,
}

impl RewriteLayer {
    pub async fn rewrite(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &'_ mut crate::flow::FlowContext,
        next: super::Next,
    ) -> Result<DynResponse, RewriteError> {
        let mut req = req;
        let (mut parts, body) = req.into_parts();
        // rewrite uri
        {
            let mut uri_parts = parts.uri.clone().into_parts();
            if let Some(host) = &self.host {
                uri_parts.authority =
                    Some(http::uri::Authority::from_maybe_shared(host.to_string())?);
            }
            if let Some(schema) = &self.schema {
                uri_parts.scheme = Some(http::uri::Scheme::try_from(schema.as_str())?);
            }
            parts.uri = http::Uri::from_parts(uri_parts)?;
        }

        if let Some(host) = &self.host {
            parts.headers.remove(http::header::HOST);
            parts
                .headers
                .insert(http::header::HOST, http::HeaderValue::from_str(host)?);
        }
        req = DynRequest::from_parts(parts, body);
        Ok(next.call(req, ctx).await)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RewriteError {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    #[error("Invalid uri: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Invalid uri parts: {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
}

impl FilterType for RewriteLayer {
    async fn call<'c>(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &'c mut crate::flow::FlowContext,
        next: super::Next,
    ) -> DynResponse {
        self.rewrite(req, ctx, next).await.unwrap_or_else(|e| {
            tracing::error!("Failed to rewrite request: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, e, ERR_FILTER_REWRITE)
        })
    }
}

pub struct Rewrite;
