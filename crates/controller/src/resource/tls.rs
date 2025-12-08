pub mod unix_local;

use switchboard_model::TlsCertParams;

pub enum TlsCertError {}

pub struct TlsCertProviderInfo {
    pub name: String,
    pub description: String,
}

pub trait TlsCertProvider {
    fn info(&self) -> TlsCertProviderInfo;
    fn discovery(&self) -> impl Future<Output = Result<TlsCertParams, TlsCertError>> + Send + '_;
}
