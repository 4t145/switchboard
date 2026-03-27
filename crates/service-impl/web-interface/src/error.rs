pub type Result<T> = ::core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("kernel error: {0}")]
    Kernel(#[from] switchboard_kernel::Error),
    #[error("bad request: {0}")]
    BadRequest(String),
}
