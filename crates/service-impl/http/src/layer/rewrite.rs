use serde::{Deserialize, Serialize};

use crate::{
    DynRequest, DynResponse,
    object::class::{ObjectClassName, SbhClass},
};

use super::{
    dynamic::SharedLayer,
    function::{FunctionLayer, Inner, LayerMethod},
};
#[derive(Clone, Deserialize, Serialize)]
pub struct RewriteLayer {
    pub host: Option<String>,
    pub schema: Option<String>,
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

impl LayerMethod for RewriteLayer {
    type Error = RewriteError;
    async fn call(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        inner: Inner,
    ) -> Result<DynResponse, Self::Error> {
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
        Ok(inner.call(req).await)
    }
}

pub struct Rewrite;
impl SbhClass for Rewrite {
    type Type = SharedLayer;
    type Error = serde_json::Error;
    fn name(&self) -> ObjectClassName {
        ObjectClassName::std("rewrite")
    }
    fn construct(&self, config: &str) -> Result<Self::Type, serde_json::Error> {
        let config: RewriteLayer = serde_json::from_str(config)?;
        Ok(SharedLayer::new(FunctionLayer::new(config)))
    }
}
