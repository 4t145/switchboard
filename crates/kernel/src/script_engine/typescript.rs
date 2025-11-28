use deno_core::JsRuntime;
use deno_core::RuntimeOptions;


pub struct TypescriptConfigRender {
    pub runtime: JsRuntime,
}

impl TypescriptConfigRender {
    pub fn new() -> Self {
        let runtime = JsRuntime::new(RuntimeOptions::default());
        Self { runtime }
    }
    async fn render_config(&mut self, script: &str) -> Result<String, super::ConfigError> {
        let result = self.runtime.call("<script>", script)
            .map_err(|_| super::ConfigError {})?;
        Ok(result.to_string())
    }
}

impl super::ConfigRender for TypescriptConfigRender {
    fn render_config(&self, script: &str) -> impl Future<Output = Result<String, super::ConfigError>> +  Send {
    }
}

use deno_ast::{MediaType, ParseParams, SourceTextInfo};

fn transpile_ts(code: &str) -> Result<String, String> {
    let parsed = deno_ast::parse_module(ParseParams {
        specifier: deno_core::ModuleSpecifier::parse("file:///config.ts").unwrap(),
        text: code.into(),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    }).map_err(|e| e.to_string())?;

    let transpiled = parsed.transpile(&Default::default(), &Default::default())
        .map_err(|e| e.to_string())?;

    Ok(transpiled.into_source().text)
}