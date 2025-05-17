use http::Request;
use hyper::{body::Body, service::Service};

pub struct RewriteService<R, S> {
    rewrite: R,
    service: S,
}

trait Rewrite {
    fn rewrite(&self, part: http::request::Parts) -> http::request::Parts;
}

impl<ReqBody: Body, R, S> Service<Request<ReqBody>> for RewriteService<R, S>
where
    S: Service<Request<ReqBody>>,
    R: Rewrite,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, req: Request<ReqBody>) -> Self::Future {
        let (part, body) = req.into_parts();
        let parts = self.rewrite.rewrite(part);
        let req = Request::from_parts(parts, body);
        self.service.call(req)
    }
}
