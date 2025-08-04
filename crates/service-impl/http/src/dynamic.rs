use bytes::Bytes;
use futures::future::{BoxFuture, TryFutureExt};
use http::{Request, Response};
use http_body_util::{BodyExt, combinators::UnsyncBoxBody};
use hyper::body::Body;
use std::{convert::Infallible, error::Error as StdError};
pub type DynBody = UnsyncBoxBody<bytes::Bytes, BoxedError>;
pub type DynRequest = Request<DynBody>;
pub type DynResponse = Response<DynBody>;
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub fn box_error<E: StdError + Send + Sync + 'static>(e: E) -> BoxedError {
    Box::new(e)
}
pub trait DynService: Send + Sync + 'static {
    fn call(&self, req: DynRequest) -> BoxFuture<'static, Result<DynResponse, Infallible>>;
}

pub trait IntoDynResponse {
    fn into_dyn_response(self) -> DynResponse;
}

impl<B> IntoDynResponse for Response<B>
where
    B: Body<Data = Bytes> + Send + 'static,
    BoxedError: From<B::Error>,
{
    fn into_dyn_response(self) -> DynResponse {
        let (parts, body) = self.into_parts();
        let body = body.map_err(BoxedError::from);
        Response::from_parts(parts, DynBody::new(body))
    }
}
