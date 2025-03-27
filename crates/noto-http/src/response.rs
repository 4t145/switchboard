use http::Response;
use hyper::body::Body;

pub trait IntoResponse {
    fn into_response(self) -> Response<impl Body>;
}
