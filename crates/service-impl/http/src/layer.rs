pub mod dynamic;
pub mod function;
pub mod rewrite;
pub mod timeout;

pub trait Layer<S> {
    type Service;
    fn layer(self, service: S) -> Self::Service;
}
