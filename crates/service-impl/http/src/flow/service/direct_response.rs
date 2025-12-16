use crate::{
    dynamic_response,
    flow::{FlowContext, node::NodeClass, service::ServiceNode},
};
use bytes::Bytes;
use http::{HeaderName, HeaderValue, StatusCode};
use http_body_util::{BodyExt, Full};
use switchboard_model::services::http::ClassId;

use crate::{DynRequest, DynResponse, box_error};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, bincode::Encode, bincode::Decode)]
pub struct DirectResponseServiceConfig {
    pub headers: Vec<(String, String)>,
    pub status_code: u16,
    pub body: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DirectResponseServiceConfigError {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[source] http::header::InvalidHeaderValue),
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[source] http::header::InvalidHeaderName),
    #[error("Invalid status code: {0}")]
    InvalidStatusCode(#[source] http::status::InvalidStatusCode),
}
#[derive(Debug, Clone)]
pub struct DirectResponseService {
    pub headers: Vec<(HeaderName, HeaderValue)>,
    pub status_code: StatusCode,
    pub body: Option<Bytes>,
}

impl DirectResponseService {
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

impl super::Service for DirectResponseService {
    fn call<'c>(
        &self,
        _: DynRequest,
        _: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + Send + 'c {
        futures::future::ready(self.make_response())
    }
}

pub struct ReverseProxyServiceClass;

impl NodeClass for ReverseProxyServiceClass {
    type Config = DirectResponseServiceConfig;
    type Error = DirectResponseServiceConfigError;
    type Node = ServiceNode<DirectResponseService>;

    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        let mut headers = Vec::new();
        for (name, value) in config.headers {
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .map_err(DirectResponseServiceConfigError::InvalidHeaderName)?;
            let header_value = HeaderValue::from_str(&value)
                .map_err(DirectResponseServiceConfigError::InvalidHeaderValue)?;
            headers.push((header_name, header_value));
        }
        let status_code = StatusCode::from_u16(config.status_code)
            .map_err(DirectResponseServiceConfigError::InvalidStatusCode)?;
        let body = config.body.map(Bytes::from);
        Ok(ServiceNode::new(DirectResponseService {
            headers,
            status_code,
            body,
        }))
    }

    fn id(&self) -> ClassId {
        ClassId::std("direct-response")
    }
}
