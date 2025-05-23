use std::{collections::BTreeMap, ops::Index};

use http::Request;
use hyper::service::Service;

use crate::router::{Route, Router, SharedRouter};

use super::dynamic::{DynRequest, SharedService};

pub struct RouterService<Req, R: Router, C: for<'a> Index<&'a Route, Output = S>, S: Service<Req>> {
    services: C,
    router: R,
    marker: std::marker::PhantomData<fn(Req)>,
}

impl<Req, R, C, S> RouterService<Req, R, C, S>
where
    R: Router,
    C: for<'a> Index<&'a Route, Output = S>,
    S: Service<Req>,
{
    pub fn new(services: C, router: R) -> Self {
        Self {
            services,
            router,
            marker: std::marker::PhantomData,
        }
    }
}

impl RouterService<DynRequest, SharedRouter, BTreeMap<Route, SharedService>, SharedService> {
    pub fn dynamic_new(services: BTreeMap<Route, SharedService>, router: SharedRouter) -> Self {
        Self {
            services,
            router,
            marker: std::marker::PhantomData,
        }
    }
}

impl<ReqBody, R, C, S> Service<Request<ReqBody>> for RouterService<Request<ReqBody>, R, C, S>
where
    R: Router,
    S: Service<Request<ReqBody>>,
    C: for<'a> Index<&'a Route, Output = S>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;
    fn call(&self, req: Request<ReqBody>) -> Self::Future {
        let (mut parts, body) = req.into_parts();
        let index = self.router.route(&mut parts);
        let service = self.services.index(&index);
        service.call(Request::from_parts(parts, body))
    }
}
