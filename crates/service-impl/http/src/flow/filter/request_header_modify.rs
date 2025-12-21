use http::{HeaderName, HeaderValue};

use serde::{Deserialize, Serialize};
use switchboard_model::services::http::ClassId;

use crate::{
    DynRequest, DynResponse,
    flow::filter::{FilterClass, FilterLike},
};

#[derive(Clone, Deserialize, Serialize, bincode::Encode, bincode::Decode)]
pub struct RequestHeaderModifyFilterConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub set: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extend: Vec<(String, String)>,
}

#[derive(Debug, thiserror::Error)]
pub enum RequestHeaderModifyFilterConfigError {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
}

impl TryInto<RequestHeaderModifyFilter> for RequestHeaderModifyFilterConfig {
    type Error = http::header::InvalidHeaderValue;

    fn try_into(self) -> Result<RequestHeaderModifyFilter, Self::Error> {
        let mut set = Vec::new();
        for (name, value) in self.set {
            let header_name = HeaderName::from_bytes(name.as_bytes()).unwrap();
            let header_value = HeaderValue::from_str(&value)?;
            set.push((header_name, header_value));
        }
        let mut remove = Vec::new();
        for name in self.remove {
            let header_name = HeaderName::from_bytes(name.as_bytes()).unwrap();
            remove.push(header_name);
        }
        let mut extend = Vec::new();
        for (name, value) in self.extend {
            let header_name = HeaderName::from_bytes(name.as_bytes()).unwrap();
            let header_value = HeaderValue::from_str(&value)?;
            extend.push((header_name, header_value));
        }
        Ok(RequestHeaderModifyFilter {
            set,
            remove,
            extend,
        })
    }
}

pub struct RequestHeaderModifyFilter {
    pub set: Vec<(HeaderName, HeaderValue)>,
    pub remove: Vec<HeaderName>,
    pub extend: Vec<(HeaderName, HeaderValue)>,
}

impl RequestHeaderModifyFilter {
    pub async fn modify(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &'_ mut crate::flow::FlowContext,
        next: super::Next,
    ) -> DynResponse {
        let mut req = req;
        let (mut parts, body) = req.into_parts();
        // modify headers
        {
            for (name, value) in &self.set {
                parts.headers.insert(name.clone(), value.clone());
            }
            for name in &self.remove {
                parts.headers.remove(name);
            }
            for (name, value) in &self.extend {
                if !parts.headers.contains_key(name) {
                    parts.headers.insert(name.clone(), value.clone());
                } else {
                    parts.headers.append(name.clone(), value.clone());
                }
            }
        }
        req = DynRequest::from_parts(parts, body);
        next.call(req, ctx).await
    }
}

impl FilterLike for RequestHeaderModifyFilter {
    async fn call(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &mut crate::flow::FlowContext,
        next: super::Next,
    ) -> DynResponse {
        self.modify(req, ctx, next).await
    }
}

pub struct RequestHeaderModifyFilterClass;

impl FilterClass for RequestHeaderModifyFilterClass {
    type Filter = RequestHeaderModifyFilter;
    type Error = RequestHeaderModifyFilterConfigError;
    type Config = RequestHeaderModifyFilterConfig;

    fn id(&self) -> ClassId {
        ClassId::std("request-header-modify")
    }

    fn construct(&self, config: Self::Config) -> Result<Self::Filter, Self::Error> {
        let filter: RequestHeaderModifyFilter = config.try_into()?;
        Ok(filter)
    }
}
