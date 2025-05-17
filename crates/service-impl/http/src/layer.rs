pub mod timeout;
pub mod function;

pub trait Layer<S> {
    type Service;
    fn layer(self, service: S) -> Self::Service;
}
