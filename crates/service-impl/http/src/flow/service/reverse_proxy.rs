use std::sync::Arc;

use crate::{
    consts::{
        ERR_REVERSE_PROXY, SERVER_NAME, X_FORWARDED_FOR, X_FORWARDED_HOST, X_FORWARDED_PROTO,
        X_REAL_IP,
    },
    extension::marker::ClientConnectionFailedMarker,
    flow::{FlowContext, node::NodeClass, service::ServiceNode},
    utils::{
        HyperHttpsClient, build_client_with_options,
        duration_expr::{self, TimeoutDuration},
        error_response,
    },
};
use http::{
    HeaderValue, Method, StatusCode, Uri, Version,
    header::{CONNECTION, UPGRADE},
    uri::{Authority, Scheme},
};
use http_body_util::BodyExt;
use hyper::ext::Protocol;
use hyper_util::rt::TokioIo;
use switchboard_model::services::http::{ClassId, consts::REVERSE_PROXY_CLASS_ID};
use tracing::{Instrument, info_span};

use crate::{DynRequest, DynResponse, box_error};
use http::header::{HOST, VIA};
const DEFAULT_UPGRADE_BUFFER_SIZE: usize = 1 << 13; // 8kb
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ReverseProxyServiceConfig {
    /// Backend authority (host:port) for proxying requests, default is empty
    pub backend: String,
    /// Scheme for backend requests, default is "http"
    pub scheme: String,
    /// Timeout duration for backend requests, default is 30 seconds
    #[serde(alias = "request_timeout")]
    pub timeout: TimeoutDuration,
    /// Pool idle timeout duration for backend requests, if not set, uses client's default (90s in hyper legacy client)
    pub pool_idle_timeout: Option<TimeoutDuration>,
    /// Whether to enforce HTTPS only connections to the backend, default is false
    pub https_only: bool,
    /// Whether to allow client upgrade http request
    pub allow_upgrade: bool,
    /// Upgrade transfer buffer size, default to be 8kb, a upgraded connection will use 2xbuffer_size space.
    pub upgrade_transfer_buffer_size: u32,
}

impl Default for ReverseProxyServiceConfig {
    fn default() -> Self {
        Self {
            backend: "".to_string(),
            scheme: "http".to_string(),
            timeout: TimeoutDuration::default(),
            pool_idle_timeout: None,
            https_only: false,
            allow_upgrade: false,
            upgrade_transfer_buffer_size: DEFAULT_UPGRADE_BUFFER_SIZE as u32,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReverseProxyServiceConfigError {
    #[error("Invalid authority: {0}")]
    InvalidAuthority(#[source] http::uri::InvalidUri),
    #[error("Invalid scheme: {0}")]
    InvalidScheme(#[source] http::uri::InvalidUri),
    #[error("HTTP client error: {0}")]
    BuildHttpClientError(#[source] std::io::Error),
}
#[derive(Debug, Clone)]
pub struct ReverseProxyService {
    pub new_authority: Authority,
    pub scheme: Scheme,
    pub client: Arc<HyperHttpsClient>,
    pub timeout: Option<std::time::Duration>,
    pub allow_upgrade: bool,
    pub upgrade_transfer_buffer_size: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ReverseProxyError {
    #[error("Invalid URI parts: {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] hyper_util::client::legacy::Error),
    #[error("Request timed out")]
    RequestTimeout { after: std::time::Duration },
}

#[derive(Debug, thiserror::Error)]
pub enum UpgradeTransferError {
    #[error("Upgrade error")]
    UpgradeError(#[from] hyper::Error),
    #[error("Tokio bidirection copy error")]
    CloneBodyError(#[from] tokio::io::Error),
}

// todo:
// extract original host properly
// add x-forwarded-proto headers

const DEFAULT_HEADER: HeaderValue = HeaderValue::from_static(SERVER_NAME);
impl ReverseProxyService {
    fn via_header_value(version: http::Version) -> HeaderValue {
        thread_local! {
            static H10_HEADER: HeaderValue = HeaderValue::from_str(&format!("1.0 {SERVER_NAME}")).expect("valid http header");
            static H11_HEADER: HeaderValue = HeaderValue::from_str(&format!("1.1 {SERVER_NAME}")).expect("valid http header");
            static H2_HEADER: HeaderValue = HeaderValue::from_str(&format!("2.0 {SERVER_NAME}")).expect("valid http header");
            static H3_HEADER: HeaderValue = HeaderValue::from_str(&format!("3.0 {SERVER_NAME}")).expect("valid http header");
        }
        match version {
            http::Version::HTTP_10 => H10_HEADER.with(HeaderValue::clone),
            http::Version::HTTP_11 => H11_HEADER.with(HeaderValue::clone),
            http::Version::HTTP_2 => H2_HEADER.with(HeaderValue::clone),
            http::Version::HTTP_3 => H3_HEADER.with(HeaderValue::clone),
            _ => DEFAULT_HEADER,
        }
    }
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
            let xff = match parts.headers.get(X_FORWARDED_FOR) {
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
            parts.headers.insert(X_FORWARDED_HOST, host);
        }

        // X-Forwarded-Proto: 原始协议
        if let Some(is_tls) = ctx.connection_info.as_ref().map(|info| info.is_tls) {
            parts.headers.insert(
                X_FORWARDED_PROTO,
                HeaderValue::from_static(if is_tls { "https" } else { "http" }),
            );
        }

        // X-Real-IP: 真实客户端 IP（仅在首次代理时设置）
        if !parts.headers.contains_key(X_REAL_IP)
            && let Some(client_ip) = ctx
                .connection_info
                .as_ref()
                .map(|info| info.peer_addr.ip().to_string())
            && let Ok(value) = HeaderValue::from_str(&client_ip)
        {
            parts.headers.insert(X_REAL_IP, value);
        }

        // Via: 代理链标识
        parts
            .headers
            .append(VIA, Self::via_header_value(parts.version));
    }
    fn process_response_parts(res_parts: &mut http::response::Parts) {
        let headers = &mut res_parts.headers;
        // 1. remove Hop-by-hop headers
        headers.remove(http::header::CONNECTION);
        headers.remove(http::header::TRANSFER_ENCODING);
        headers.remove(http::header::UPGRADE);
        headers.remove(http::header::TRAILER);
        headers.remove(http::header::TE);

        // 2. add Server header
        headers.insert(
            http::header::SERVER,
            HeaderValue::from_static(SERVER_NAME), // e.g., "switchboard"
        );

        // 3. add Via header
        headers.append(http::header::VIA, Self::via_header_value(res_parts.version));
    }
    pub async fn call_inner(self, req: DynRequest) -> Result<DynResponse, ReverseProxyError> {
        // check upgrade
        let need_upgrade = self.allow_upgrade
            && (false
                // is h1 upgrade
                || (req.version() == Version::HTTP_11
                    && req.headers().get(UPGRADE).is_some()
                    && req
                        .headers()
                        .get_all(CONNECTION)
                        .iter()
                        .any(|h| h == HeaderValue::from_name(UPGRADE)))
                // is h2 websocket/
                || (req.version() == Version::HTTP_2
                    && req.method() == Method::CONNECT
                    && req
                        .extensions()
                        .get::<Protocol>()
                        .is_some_and(|p| p.as_str() == "websocket" || p.as_str() == "webtransport")));
        let mut req = {
            let (mut parts, body) = req.into_parts();
            let mut uri_parts = parts.uri.into_parts();
            uri_parts.authority = Some(self.new_authority);
            uri_parts.scheme = Some(self.scheme);
            parts.uri = Uri::from_parts(uri_parts)?;
            DynRequest::from_parts(parts, body)
        };
        if need_upgrade {
            let request_upgrade = hyper::upgrade::on(&mut req);
            let req_fut = self.client.request(req);
            let response = if let Some(request_timeout) = self.timeout {
                tokio::time::timeout(request_timeout, req_fut)
                    .await
                    .map_err(|_| ReverseProxyError::RequestTimeout {
                        after: request_timeout,
                    })??
            } else {
                req_fut.await?
            };
            let (mut resp_parts, body) = response.into_parts();
            let body = body.map_err(box_error).boxed_unsync();
            let need_upgrade_response = false
                || (resp_parts.version == Version::HTTP_11
                    && resp_parts.status == StatusCode::SWITCHING_PROTOCOLS)
                || (resp_parts.version == Version::HTTP_2 && resp_parts.status.is_success());
            if !need_upgrade_response {
                Self::process_response_parts(&mut resp_parts);
                return Ok(DynResponse::from_parts(resp_parts, body));
            }
            let mut response = DynResponse::from_parts(resp_parts, body);
            let response_upgrade = hyper::upgrade::on(&mut response);
            let upgrade_transfer_buffer_size = self.upgrade_transfer_buffer_size;
            tokio::spawn(
                async move {
                    let (request_upgrade, response_upgrade) =
                        tokio::try_join!(request_upgrade, response_upgrade)?;
                    let (upload, download) = tokio::io::copy_bidirectional_with_sizes(
                        &mut TokioIo::new(request_upgrade),
                        &mut TokioIo::new(response_upgrade),
                        upgrade_transfer_buffer_size,
                        upgrade_transfer_buffer_size,
                    )
                    .await?;
                    tracing::debug!(upload, download, "transfer finished");
                    Ok::<(), UpgradeTransferError>(())
                }
                .instrument(info_span!("reverse_proxy.upgrade")),
            );
            Ok(response)
        } else {
            // plain http
            let req_fut = self.client.request(req);
            let response = if let Some(request_timeout) = self.timeout {
                tokio::time::timeout(request_timeout, req_fut)
                    .await
                    .map_err(|_| ReverseProxyError::RequestTimeout {
                        after: request_timeout,
                    })??
            } else {
                req_fut.await?
            };
            let (mut resp_parts, body) = response.into_parts();
            let body = body.map_err(box_error).boxed_unsync();
            Self::process_response_parts(&mut resp_parts);
            Ok(DynResponse::from_parts(resp_parts, body))
        }
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
                Err(ReverseProxyError::RequestTimeout { after }) => error_response(
                    StatusCode::GATEWAY_TIMEOUT,
                    format!(
                        "Request timed out after {}",
                        duration_expr::DurationExprDisplay(&after)
                    ),
                    ERR_REVERSE_PROXY,
                ),
                Err(ReverseProxyError::HttpClientError(e)) => {
                    let is_connection_error = e.is_connect();
                    let mut response =
                        error_response(StatusCode::BAD_GATEWAY, e, ERR_REVERSE_PROXY);
                    if is_connection_error {
                        response
                            .extensions_mut()
                            .insert(ClientConnectionFailedMarker);
                    }
                    response
                }
                Err(e) => error_response(StatusCode::BAD_GATEWAY, e, ERR_REVERSE_PROXY),
            }
        }
    }
}

pub struct ReverseProxyServiceClass;

impl NodeClass for ReverseProxyServiceClass {
    type Config = ReverseProxyServiceConfig;
    type Error = ReverseProxyServiceConfigError;
    type Node = ServiceNode<ReverseProxyService>;

    fn construct(&self, config: ReverseProxyServiceConfig) -> Result<Self::Node, Self::Error> {
        let authority = config
            .backend
            .parse()
            .map_err(ReverseProxyServiceConfigError::InvalidAuthority)?;
        let timeout = config.timeout.as_duration();
        let client_options = crate::utils::ClientOptions {
            pool_idle_timeout: config.pool_idle_timeout,
            https_only: config.https_only,
        };
        Ok(ServiceNode::new(ReverseProxyService {
            new_authority: authority,
            scheme: config
                .scheme
                .parse()
                .map_err(ReverseProxyServiceConfigError::InvalidScheme)?,
            client: Arc::new(
                build_client_with_options(client_options)
                    .map_err(ReverseProxyServiceConfigError::BuildHttpClientError)?,
            ),
            timeout,
            allow_upgrade: config.allow_upgrade,
            upgrade_transfer_buffer_size: config.upgrade_transfer_buffer_size as usize,
        }))
    }

    fn id(&self) -> ClassId {
        ClassId::std(REVERSE_PROXY_CLASS_ID)
    }
}
