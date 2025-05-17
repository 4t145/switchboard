use hyper::service::{Service, service_fn};

pub struct FunctionService<F> {
    pub function: F,
}

impl<F, Req, Resp, Err, Fut> Service<Req> for FunctionService<F>
where
    F: Fn(Req) -> Fut,
    Fut: Future<Output = Result<Resp, Err>>,
{
    type Future = Fut;
    type Error = Err;
    type Response = Resp;

    fn call(&self, req: Req) -> Self::Future {
        (self.function)(req)
    }
}
