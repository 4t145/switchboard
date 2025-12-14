use http::header::HOST;

#[derive(Debug, thiserror::Error)]
pub enum BadHostnameError {
    #[error("Invalid :authority header: {0}")]
    InvalidAuthorityHeader(#[source] http::header::ToStrError),
    #[error("Invalid host header: {0}")]
    InvalidHostHeader(#[source] http::header::ToStrError),
    #[error("No hostname found in request")]
    NoHostname,
}
pub fn try_extract_hostname(parts: &http::request::Parts) -> Result<&str, BadHostnameError> {
    if let Some(header) = parts.headers.get(":authority") {
        return header
            .to_str()
            .map_err(BadHostnameError::InvalidAuthorityHeader);
    }
    if let Some(header) = parts.headers.get(HOST) {
        return header.to_str().map_err(BadHostnameError::InvalidHostHeader);
    }
    // check uri:authority
    if let Some(authority) = parts.uri.authority() {
        return Ok(authority.host());
    }
    Err(BadHostnameError::NoHostname)
}
