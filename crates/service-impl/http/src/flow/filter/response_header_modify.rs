use http::{HeaderName, HeaderValue};

use serde::{Deserialize, Serialize};
use switchboard_model::services::http::ClassId;

use crate::{
    DynRequest, DynResponse,
    flow::filter::{FilterClass, FilterLike},
};

#[derive(Clone, Deserialize, Serialize, bincode::Encode, bincode::Decode)]
pub struct ResponseHeaderModifyFilterConfig {
    pub set: Vec<(String, String)>,
    pub remove: Vec<String>,
    pub extend: Vec<(String, String)>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResponseHeaderModifyFilterConfigError {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
}

impl TryInto<ResponseHeaderModifyFilter> for ResponseHeaderModifyFilterConfig {
    type Error = http::header::InvalidHeaderValue;

    fn try_into(self) -> Result<ResponseHeaderModifyFilter, Self::Error> {
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
        Ok(ResponseHeaderModifyFilter {
            set,
            remove,
            extend,
        })
    }
}

pub struct ResponseHeaderModifyFilter {
    pub set: Vec<(HeaderName, HeaderValue)>,
    pub remove: Vec<HeaderName>,
    pub extend: Vec<(HeaderName, HeaderValue)>,
}

impl ResponseHeaderModifyFilter {
    pub async fn modify(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &'_ mut crate::flow::FlowContext,
        next: super::Next,
    ) -> DynResponse {
        let resonse = next.call(req, ctx).await;
        let (mut parts, body) = resonse.into_parts();
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
        DynResponse::from_parts(parts, body)
    }
}

impl FilterLike for ResponseHeaderModifyFilter {
    async fn call<'c>(
        self: std::sync::Arc<Self>,
        req: DynRequest,
        ctx: &'c mut crate::flow::FlowContext,
        next: super::Next,
    ) -> DynResponse {
        self.modify(req, ctx, next).await
    }
}

pub struct ResponseHeaderModifyFilterClass;

impl FilterClass for ResponseHeaderModifyFilterClass {
    type Filter = ResponseHeaderModifyFilter;
    type Error = ResponseHeaderModifyFilterConfigError;
    type Config = ResponseHeaderModifyFilterConfig;

    fn id(&self) -> ClassId {
        ClassId::std("response-header-modify")
    }

    fn construct(&self, config: Self::Config) -> Result<Self::Filter, Self::Error> {
        let filter: ResponseHeaderModifyFilter = config.try_into()?;
        Ok(filter)
    }
}
