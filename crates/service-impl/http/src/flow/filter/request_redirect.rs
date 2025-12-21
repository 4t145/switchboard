use bytes::Bytes;
use http::{StatusCode, Uri};
use http_body_util::Full;

use crate::{
    DynResponse, dynamic_response,
    flow::filter::{FilterClass, FilterLike},
};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, bincode::Encode, bincode::Decode)]
#[serde(default)]
pub struct RequestRedirectFilterConfig {
    /// target URL to redirect to
    pub to: String,
    /// expected to be a 3xx status code
    pub status_code: u16,
    /// optional content body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl Default for RequestRedirectFilterConfig {
    fn default() -> Self {
        Self {
            to: "/".to_string(),
            status_code: 301,
            content: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RequestRedirectFilter {
    pub to: Uri,
    pub status_code: StatusCode,
    pub content: Option<Bytes>,
}

#[derive(Debug, thiserror::Error)]
pub enum RequestRedirectFilterConfigError {
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("Invalid status code: {0}")]
    InvalidStatusCode(#[from] http::status::InvalidStatusCode),
    #[error("Unsupported status code: {0}")]
    UnsupportedStatusCode(u16),
}

impl RequestRedirectFilter {
    pub fn into_response(&self) -> DynResponse {
        let response = http::Response::builder()
            .status(self.status_code)
            .header(http::header::LOCATION, self.to.to_string());
        dynamic_response(
            response
                .body(Full::new(self.content.clone().unwrap_or_default()))
                .expect("should be valid response"),
        )
    }
}

impl FilterLike for RequestRedirectFilter {
    async fn call(
        self: std::sync::Arc<Self>,
        _req: crate::DynRequest,
        _ctx: &mut crate::flow::FlowContext,
        _next: super::Next,
    ) -> DynResponse {
        self.into_response()
    }
}

pub struct RequestRedirectFilterClass;

impl FilterClass for RequestRedirectFilterClass {
    type Filter = RequestRedirectFilter;
    type Error = RequestRedirectFilterConfigError;
    type Config = RequestRedirectFilterConfig;

    fn id(&self) -> switchboard_model::services::http::ClassId {
        switchboard_model::services::http::ClassId::std("request-redirect")
    }

    fn construct(&self, config: Self::Config) -> Result<Self::Filter, Self::Error> {
        let to = config.to.parse::<Uri>()?;
        let status_code = StatusCode::from_u16(config.status_code)?;
        if !status_code.is_redirection() {
            return Err(RequestRedirectFilterConfigError::UnsupportedStatusCode(
                config.status_code,
            ));
        }
        Ok(RequestRedirectFilter {
            to,
            status_code,
            content: config.content.map(Bytes::from),
        })
    }
}
