use bytes::Bytes;
use http::{
    StatusCode,
    header::{CONTENT_TYPE, SERVER},
};
use http_body_util::BodyExt;

use crate::{DynResponse, box_error, consts::SERVER_NAME};
pub const HEADER_X_SBH_ERROR: &str = "X-Switchboard-Error";
pub fn error_response(
    code: StatusCode,
    error: impl std::fmt::Display,
    kind: &'static str,
) -> DynResponse {
    let body = error.to_string();
    let body = http_body_util::Full::<Bytes>::from(body)
        .map_err(box_error)
        .boxed_unsync();
    http::Response::builder()
        .status(code)
        .header(CONTENT_TYPE, "text/plain; charset=UTF-8")
        .header(HEADER_X_SBH_ERROR, kind)
        .header(SERVER, SERVER_NAME)
        .body(body)
        .unwrap()
}
