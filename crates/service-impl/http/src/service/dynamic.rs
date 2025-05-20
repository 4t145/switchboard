use bytes::Bytes;
use futures::future::{BoxFuture, TryFutureExt};
use http::{Request, Response};
use http_body_util::{BodyExt, combinators::UnsyncBoxBody};
use hyper::{body::Body, service::Service};
use std::{convert::Infallible, error::Error as StdError, sync::Arc};
pub type DynBody = UnsyncBoxBody<bytes::Bytes, BoxedError>;
pub type DynRequest = Request<DynBody>;
pub type DynResponse = Response<DynBody>;
pub type BoxedError = Box<dyn std::error::Error + Send + 'static>;
pub fn dyn_error<E: StdError + Send + 'static>(e: E) -> BoxedError {
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
    B::Error: StdError + Send,
{
    fn into_dyn_response(self) -> DynResponse {
        let (parts, body) = self.into_parts();
        let body = body.map_err(dyn_error);
        Response::from_parts(parts, DynBody::new(body))
    }
}

impl<S> DynService for S
where
    S: Service<Request<DynBody>, Error = Infallible> + Send + Sync + 'static,
    S::Response: IntoDynResponse,
    S::Future: Send + 'static,
{
    fn call(&self, req: DynRequest) -> BoxFuture<'static, Result<DynResponse, Infallible>> {
        Box::pin(
            <Self as Service<Request<DynBody>>>::call(self, req)
                .map_ok(IntoDynResponse::into_dyn_response),
        )
    }
}
#[derive(Clone)]
pub struct SharedService {
    service: Arc<dyn DynService>,
}

impl<ReqBody> Service<Request<ReqBody>> for SharedService
where
    ReqBody: Body<Data = Bytes> + Send + 'static,
    ReqBody::Error: StdError + Send + 'static,
{
    type Response = Response<DynBody>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<ReqBody>) -> Self::Future {
        let req = req.map(|body| DynBody::new(body.map_err(dyn_error)));
        let service = self.service.clone();
        service.call(req)
    }
}

impl SharedService {
    pub fn new<S>(service: S) -> Self
    where
        S: DynService,
    {
        Self {
            service: Arc::new(service),
        }
    }
}

