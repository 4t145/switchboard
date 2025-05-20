use std::ops::Index;

use http::Request;
use hyper::service::Service;

use crate::router::{Route, Router};

pub struct RouterService<Req, R: Router, C: Index<Route, Output = S>, S: Service<Req>> {
    services: C,
    router: R,
    marker: std::marker::PhantomData<fn(Req)>,
}

impl<ReqBody, R, C, S> Service<Request<ReqBody>> for RouterService<Request<ReqBody>, R, C, S>
where
    R: Router,
    S: Service<Request<ReqBody>>,
    C: Index<Route, Output = S>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;
    fn call(&self, req: Request<ReqBody>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let index = self.router.route(&parts);
        let service = self.services.index(index);
        service.call(Request::from_parts(parts, body))
    }
}
