#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("fail to extract hostname: {0}")]
    HostExtractFailed(#[from] crate::utils::hostname::BadHostnameError),
    #[error("no matching route found")]
    NoMatchRoute,
    #[error("no matching host found")]
    HostNotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("matchit insert error: {0}")]
    MatchitInsertError(#[from] matchit::InsertError),
    #[error("invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("invalid http method: {0}")]
    InvalidHttpMethod(#[from] http::method::InvalidMethod),
}
