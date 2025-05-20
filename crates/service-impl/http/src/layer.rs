pub mod timeout;
pub mod function;
pub mod take_error;
pub mod dynamic;

pub trait Layer<S> {
    type Service;
    fn layer(self, service: S) -> Self::Service;
}
