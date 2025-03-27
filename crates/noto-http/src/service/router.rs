use std::{hash::Hash, ops::Index};

use hyper::service::Service;

pub trait Router<Req> {
    type Index: Hash + Eq + Ord;
    fn route(&self, req: &Req) -> Self::Index;
}

pub struct RouterService<Req, R: Router<Req>, C: Index<R::Index, Output = S>, S: Service<Req>> {
    services: C,
    router: R,
    marker: std::marker::PhantomData<fn(Req)>,
}

impl<Req, R, C, S> Service<Req> for RouterService<Req, R, C, S>
where
    R: Router<Req>,
    S: Service<Req>,
    C: Index<R::Index, Output = S>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;
    fn call(&self, req: Req) -> Self::Future {
        let index = self.router.route(&req);
        let service = self.services.index(index);
        service.call(req)
    }
}
