use std::convert::Infallible;

use futures::future::BoxFuture;
use http::StatusCode;

use crate::utils::error_response;

use super::dynamic::{DynRequest, DynResponse, DynService};

pub struct FunctionService<F, E, Fut> {
    pub function: F,
    pub error_kind: &'static str,
    pub _marker: std::marker::PhantomData<(fn() -> E, fn() -> Fut)>,
}

impl<F, E, Fut> FunctionService<F, E, Fut>
where
    F: Fn(DynRequest) -> Fut + Send + Sync + 'static + Clone,
    E: std::error::Error + Send + Sync + 'static,
    Fut: Future<Output = Result<DynResponse, E>> + Send + 'static,
{
    pub fn new(function: F, error_kind: &'static str) -> Self {
        Self {
            function,
            error_kind,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, E, Fut> DynService for FunctionService<F, E, Fut>
where
    F: Fn(DynRequest) -> Fut + Send + Sync + 'static + Clone,
    E: std::error::Error + Send + Sync + 'static,
    Fut: Future<Output = Result<DynResponse, E>> + Send + 'static,
{
    fn call(
        &self,
        req: super::dynamic::DynRequest,
    ) -> BoxFuture<'static, Result<DynResponse, Infallible>> {
        let function = self.function.clone();
        let error_kind = self.error_kind;
        Box::pin(async move {
            match function(req).await {
                Ok(response) => Ok(response),
                Err(error) => {
                    let response =
                        error_response(StatusCode::INTERNAL_SERVER_ERROR, error, error_kind);
                    Ok(response)
                }
            }
        })
    }
}
