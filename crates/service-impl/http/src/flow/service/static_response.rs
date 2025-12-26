use crate::{
    dynamic_response,
    flow::{FlowContext, node::NodeClass, service::ServiceNode},
};
use bytes::Bytes;
use http::{HeaderName, HeaderValue, StatusCode};
use http_body_util::{BodyExt, Full};
use switchboard_model::services::http::{ClassId, consts::STATIC_RESPONSE_CLASS_ID};

use crate::{DynRequest, DynResponse, box_error};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, bincode::Encode, bincode::Decode)]
pub struct StaticResponseServiceConfig {
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default = "default_status_code", alias = "code")]
    pub status_code: u16,
    #[serde(default)]
    pub body: Option<String>,
}

fn default_status_code() -> u16 {
    404
}

#[derive(Debug, thiserror::Error)]
pub enum StaticResponseServiceConfigError {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[source] http::header::InvalidHeaderValue),
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[source] http::header::InvalidHeaderName),
    #[error("Invalid status code: {0}")]
    InvalidStatusCode(#[source] http::status::InvalidStatusCode),
}
#[derive(Debug, Clone)]
pub struct StaticResponseService {
    pub headers: Vec<(HeaderName, HeaderValue)>,
    pub status_code: StatusCode,
    pub body: Option<Bytes>,
}

impl StaticResponseService {
    pub fn make_response(&self) -> DynResponse {
        let body = match &self.body {
            Some(b) => b.clone(),
            None => Bytes::new(),
        };
        let response = http::Response::builder()
            .status(self.status_code)
            .body(Full::new(body).map_err(box_error))
            .expect("building response should not fail");
        let mut response = dynamic_response(response);
        response.headers_mut().extend(self.headers.clone());
        response
    }
}

impl super::Service for StaticResponseService {
    fn call<'c>(
        &self,
        _: DynRequest,
        _: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + Send + 'c {
        futures::future::ready(self.make_response())
    }
}

pub struct StaticResponseServiceClass;

impl NodeClass for StaticResponseServiceClass {
    type Config = StaticResponseServiceConfig;
    type Error = StaticResponseServiceConfigError;
    type Node = ServiceNode<StaticResponseService>;

    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        let mut headers = Vec::new();
        for (name, value) in config.headers {
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .map_err(StaticResponseServiceConfigError::InvalidHeaderName)?;
            let header_value = HeaderValue::from_str(&value)
                .map_err(StaticResponseServiceConfigError::InvalidHeaderValue)?;
            headers.push((header_name, header_value));
        }
        let status_code = StatusCode::from_u16(config.status_code)
            .map_err(StaticResponseServiceConfigError::InvalidStatusCode)?;
        let body = config.body.map(Bytes::from);
        Ok(ServiceNode::new(StaticResponseService {
            headers,
            status_code,
            body,
        }))
    }

    fn id(&self) -> ClassId {
        ClassId::std(STATIC_RESPONSE_CLASS_ID)
    }
}
