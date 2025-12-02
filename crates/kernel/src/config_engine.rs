use std::collections::HashMap;

pub mod typescript;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing correspondent renderer for language: {lang}")]
    MissingCorrespondedRenderer { lang: String },
    #[error("typescript error: {0}")]
    TypescriptError(#[from] typescript::TypescriptError),
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("toml deserialization error: {0}")]
    TomlDeserializationError(#[from] toml::de::Error),
}

pub trait ConfigRender {
    fn render_config(
        &self,
        script: &str,
    ) -> impl Future<Output = Result<String, ConfigError>> + Send;
}

pub struct ConfigRenderRouter {
    pub typescript_renderer: Option<typescript::TypescriptConfigRender>,
}

impl ConfigRenderRouter {
    pub fn new_full_featured() -> Self {
        Self {
            typescript_renderer: Some(typescript::TypescriptConfigRender::spawn()),
        }
    }

    pub async fn render_config(
        &self,
        lang: &str,
        raw: &str,
    ) -> Result<switchboard_model::Config, ConfigError> {
        match lang {
            "typescript" => {
                if let Some(renderer) = &self.typescript_renderer {
                    let result = renderer.render_config(raw).await?;
                    let config: switchboard_model::Config = serde_json::from_str(&result)?;
                    Ok(config)
                } else {
                    Err(ConfigError::MissingCorrespondedRenderer {
                        lang: lang.to_string(),
                    })
                }
            }
            "json" => Ok(serde_json::from_str(raw)?),
            "toml" => Ok(toml::from_str(raw)?),
            _ => Err(ConfigError::MissingCorrespondedRenderer {
                lang: lang.to_string(),
            }),
        }
    }
}
