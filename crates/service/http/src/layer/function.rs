// use hyper::service::Service;

// use crate::service::function::FunctionService;

// use super::Layer;

// pub struct FunctionLayer<Req, Resp, Err, Fut, F> 

// {
//     f: F,

// }

// pub struct Inner<S>(S);

// impl<Req, Resp, Err, Fut, F, S> Layer<S> for FunctionLayer<Req, Resp, Err, Fut, F>
// where
//     F: Fn(Req, &S) -> Fut + 'static,
//     Fut: Future<Output = Result<Resp, Err>>,
//     S: Service<Req> + 'static,
// {
//     type Service = FunctionService<Box<dyn Fn(Req) -> Fut>>;

//     fn layer(self, service: S) -> Self::Service {
//         FunctionService {
//             function: Box::new(move |req| (self.f)(req, &service)),
//         }
//     }
// }
