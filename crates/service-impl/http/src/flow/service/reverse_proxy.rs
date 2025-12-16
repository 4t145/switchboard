use std::sync::Arc;

use crate::{
    ERR_HTTP_CLIENT,
    flow::{FlowContext, node::NodeClass, service::ServiceNode},
    utils::{HyperHttpsClient, build_client, error_response},
};
use http::{HeaderValue, StatusCode, Uri, uri::Authority};
use http_body_util::BodyExt;
use switchboard_model::services::http::ClassId;

use crate::{DynRequest, DynResponse, box_error};
use http::header::{HOST, VIA};
pub const X_FORWARDED_FOR: &str = "x-forwarded-for";
pub const X_FORWARDED_HEADERS: &str = "x-forwarded-headers";
pub const X_REAL_IP: &str = "x-real-ip";

#[derive(Debug, Clone)]
#[derive(serde::Deserialize, serde::Serialize, bincode::Encode, bincode::Decode)]
pub struct ReverseProxyServiceConfig {
    pub new_authority: String,
    pub schema: String,
}


#[derive(Debug, thiserror::Error)]
pub enum ReverseProxyServiceConfigError {
    #[error("Invalid authority: {0}")]
    InvalidAuthority(#[from] http::uri::InvalidUri),
    #[error("HTTP client error: {0}")]
    BuildHttpClientError(#[source] std::io::Error),
}
#[derive(Debug, Clone)]
pub struct ReverseProxyService {
    pub new_authority: Authority,
    pub schema: Arc<str>,
    pub client: HyperHttpsClient,
}

#[derive(Debug, thiserror::Error)]
pub enum ReverseProxyError {
    #[error("Invalid URI parts: {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] hyper_util::client::legacy::Error),
}

// todo: 
// extract original host properly
// add x-forwarded-proto headers

impl ReverseProxyService {
    fn add_forwarded_headers(
        &self,
        parts: &mut http::request::Parts,
        ctx: &FlowContext,
        original_host: Option<HeaderValue>,
    ) {
        // X-Forwarded-For: 追加客户端 IP
        if let Some(client_ip) = ctx
            .connection_info
            .as_ref()
            .map(|info| info.peer_addr.ip().to_string())
        {
            let xff = match parts.headers.get("x-forwarded-for") {
                Some(existing) => {
                    let existing_str = existing.to_str().unwrap_or("");
                    format!("{}, {}", existing_str, client_ip)
                }
                None => client_ip,
            };
            if let Ok(value) = HeaderValue::from_str(&xff) {
                parts.headers.insert(X_FORWARDED_FOR, value);
            }
        }

        // X-Forwarded-Host: 原始 Host
        if let Some(host) = original_host {
            parts.headers.insert("x-forwarded-host", host);
        }

        // X-Forwarded-Proto: 原始协议
        // let proto = if ctx.is_tls { "https" } else { "http" };
        // if let Ok(value) = HeaderValue::from_static(proto) {
        //     parts.headers.insert("x-forwarded-proto", value);
        // }

        // X-Real-IP: 真实客户端 IP（仅在首次代理时设置）
        if !parts.headers.contains_key(X_REAL_IP) {
            if let Some(client_ip) = ctx
                .connection_info
                .as_ref()
                .map(|info| info.peer_addr.ip().to_string())
            {
                if let Ok(value) = HeaderValue::from_str(&client_ip) {
                    parts.headers.insert(X_REAL_IP, value);
                }
            }
        }

        // Via: 代理链标识
        let version = match parts.version {
            http::Version::HTTP_10 => "1.0",
            http::Version::HTTP_11 => "1.1",
            http::Version::HTTP_2 => "2.0",
            http::Version::HTTP_3 => "3.0",
            _ => "unknown",
        };
        let via = format!("{} switchboard", version);
        if let Ok(value) = HeaderValue::from_str(&via) {
            parts.headers.append(VIA, value);
        }
    }
    pub async fn call_inner(self, req: DynRequest) -> Result<DynResponse, ReverseProxyError> {
        let req = {
            let (mut parts, body) = req.into_parts();
            let mut uri_parts = parts.uri.into_parts();
            uri_parts.authority = Some(self.new_authority);
            parts.uri = Uri::from_parts(uri_parts)?;
            DynRequest::from_parts(parts, body)
        };
        let response = self.client.request(req).await?;
        Ok(response.map(|incoming| incoming.map_err(box_error).boxed_unsync()))
    }
}

impl super::Service for ReverseProxyService {
    fn call<'c>(
        &self,
        req: DynRequest,
        ctx: &'c mut FlowContext,
    ) -> impl Future<Output = DynResponse> + Send + 'c {
        let this = self.clone();
        let req = {
            let (mut parts, body) = req.into_parts();
            let original_host = parts.headers.get(HOST).cloned();
            self.add_forwarded_headers(&mut parts, ctx, original_host);
            DynRequest::from_parts(parts, body)
        };
        async move {
            match this.call_inner(req).await {
                Ok(response) => response,
                Err(e) => error_response(StatusCode::BAD_GATEWAY, e, ERR_HTTP_CLIENT),
            }
        }
    }
}


pub struct ReverseProxyServiceClass;

impl NodeClass for ReverseProxyServiceClass {
    type Config = ReverseProxyServiceConfig;
    type Error = ReverseProxyServiceConfigError;
    type Node = ServiceNode<ReverseProxyService>;

    fn construct(&self, config: Self::Config) -> Result<Self::Node, Self::Error> {
        let authority = config
            .new_authority
            .parse()?;
        Ok(ServiceNode::new(ReverseProxyService {
            new_authority: authority,
            schema: Arc::from(config.schema),
            client: build_client().map_err(ReverseProxyServiceConfigError::BuildHttpClientError)?,
        }))
    }

    fn id(&self) -> ClassId {
        ClassId::std("reverse-proxy")
    }
}
