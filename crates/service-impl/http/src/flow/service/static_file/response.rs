use crate::{empty_body, DynResponse};
use http::header::{CACHE_CONTROL, ETAG, LAST_MODIFIED};
use http::StatusCode;

pub fn empty_response(code: StatusCode) -> DynResponse {
    let mut response = DynResponse::new(empty_body());
    *response.status_mut() = code;
    response
}

pub fn not_found() -> DynResponse {
    empty_response(StatusCode::NOT_FOUND)
}

pub fn forbidden() -> DynResponse {
    empty_response(StatusCode::FORBIDDEN)
}

pub fn not_modified_from(response: &DynResponse) -> DynResponse {
    let mut out = DynResponse::new(empty_body());
    *out.status_mut() = StatusCode::NOT_MODIFIED;

    for name in [CACHE_CONTROL, ETAG, LAST_MODIFIED] {
        if let Some(value) = response.headers().get(&name).cloned() {
            out.headers_mut().insert(name, value);
        }
    }

    out
}
