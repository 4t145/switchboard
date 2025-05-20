use std::convert::Infallible;

use futures_core::future::BoxFuture;
use http::Request;
use hyper::service::Service;

pub type BoxedBody = http_body_util::combinators::UnsyncBoxBody<bytes::Bytes, hyper::Error>;
pub type DynRequest = http::Request<BoxedBody>;
pub type DynResponse = http::Response<BoxedBody>;
pub type BoxedError = Box<dyn std::error::Error + Send + 'static>;

pub trait DynService {
    fn call(&self, req: DynRequest) -> BoxFuture<'static, Result<DynResponse, Infallible>>;
}
