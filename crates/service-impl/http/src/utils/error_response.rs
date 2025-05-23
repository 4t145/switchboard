use std::error::Error as StdError;

use bytes::Bytes;
use http::{StatusCode, header::CONTENT_TYPE};
use http_body_util::BodyExt;

use crate::service::dynamic::{DynResponse, box_error};
pub const HEADER_X_SBH_ERROR: &str = "X-Sbh-Error";
pub fn error_response(code: StatusCode, error: impl StdError, kind: &'static str) -> DynResponse {
    let body = error.to_string();
    let body = http_body_util::Full::<Bytes>::from(body)
        .map_err(box_error)
        .boxed_unsync();
    http::Response::builder()
        .status(code)
        .header(CONTENT_TYPE, "text/plain; charset=UTF-8")
        .header(HEADER_X_SBH_ERROR, kind)
        .body(body)
        .unwrap()
}
