use bytes::Bytes;
use http::{Request, Response};
use http_body_util::{BodyExt, combinators::UnsyncBoxBody};
use hyper::body::Body;
use std::error::Error as StdError;
pub type DynBody = UnsyncBoxBody<bytes::Bytes, BoxedError>;
pub type DynRequest = Request<DynBody>;
pub type DynResponse = Response<DynBody>;
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub fn box_error<E: StdError + Send + Sync + 'static>(e: E) -> BoxedError {
    Box::new(e)
}
pub trait IntoDynResponse {
    fn into_dyn_response(self) -> DynResponse;
}

pub fn dynamic_response<B>(response: Response<B>) -> DynResponse
where
    B: Body<Data = Bytes> + Send + 'static,
    BoxedError: From<B::Error>,
{
    let (parts, body) = response.into_parts();
    let body = body.map_err(BoxedError::from);
    Response::from_parts(parts, DynBody::new(body))
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

/// Clone the request body for cases where it needs to be read multiple times.
///
/// # Warnings
/// Cloning the body requires reading it fully into memory, which can lead to
/// high memory usage for large bodies. Use with caution.
///
/// # Errors
/// Returns an error if reading the body fails.
pub async fn clone_body(body: &mut DynBody) -> Result<DynBody, BoxedError> {
    let collected_body = body.collect().await?;
    let cloned_body = DynBody::new(collected_body.map_err(box_error));
    Ok(cloned_body)
}

pub fn empty_body() -> DynBody {
    DynBody::new(http_body_util::Empty::<bytes::Bytes>::new().map_err(box_error))
}
