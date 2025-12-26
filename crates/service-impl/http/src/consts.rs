pub const SERVER_NAME: &str = "switchboard";
pub const FORKED_MARKER_HEADER: &str = "x-switchboard-forked";

pub const ERR_HTTP_CLIENT: &str = "service.http-client";
pub const ERR_REVERSE_PROXY: &str = "service.reverse-proxy";
pub const ERR_FLOW: &str = "flow";

pub const ERROR_BALANCER: &str = "balancer";
// FILTER ERRORS
pub const ERR_FILTER_URL_REWRITE: &str = "filter.url-rewrite";
pub const ERR_FILTER_REQUEST_MIRROR: &str = "filter.request-mirror";
pub const ERR_FILTER_REQUEST_HEADER_MODIFY: &str = "filter.request-header-modify";
pub const ERR_FILTER_RESPONSE_HEADER_MODIFY: &str = "filter.response-header-modify";

// headers
pub const X_FORWARDED_FOR: &str = "x-forwarded-for";
pub const X_FORWARDED_HEADERS: &str = "x-forwarded-headers";
pub const X_FORWARDED_HOST: &str = "x-forwarded-host";
pub const X_FORWARDED_PROTO: &str = "x-forwarded-proto";
pub const X_REAL_IP: &str = "x-real-ip";
