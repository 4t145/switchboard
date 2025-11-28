pub mod typescript;


pub enum ConfigError {

}

pub trait ConfigRender {
    fn render_config(&self, script: &str) -> impl Future<Output = Result<String, ConfigError>> +  Send;
}