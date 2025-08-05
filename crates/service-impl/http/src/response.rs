use bytes::Bytes;
use http::Response;
use hyper::body::Body;

pub trait IntoResponse {
    type Error: std::error::Error + Send + Sync + 'static;
    fn into_response(self) -> Response<impl Body<Data = Bytes, Error = Self::Error>>;
}
