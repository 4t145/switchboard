pub mod typescript;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("typescript error: {0}")]
    TypescriptError(#[from] typescript::TypescriptError),
}

pub trait ConfigRender {
    fn render_config(&self, script: &str) -> impl Future<Output = Result<String, ConfigError>> +  Send;
}