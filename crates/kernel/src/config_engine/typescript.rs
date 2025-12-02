use deno_ast::EmitOptions;
use deno_ast::EmittedSourceText;
use deno_ast::ParseDiagnostic;
use deno_ast::TranspileError;
use deno_ast::TranspileModuleOptions;
use deno_ast::TranspileOptions;
use deno_ast::{MediaType, ParseParams};
use deno_core::JsRuntime;
use deno_core::PollEventLoopOptions;
use deno_core::RuntimeOptions;
use deno_core::serde_v8;
#[derive(Debug, thiserror::Error)]
pub enum TypescriptError {
    #[error("parse diagnostic: {0}")]
    ParseDiagnostic(#[from] ParseDiagnostic),
    #[error("transpile error: {0}")]
    TranspileError(#[from] TranspileError),
    #[error("serde_v8 error: {0}")]
    SerdeV8Error(#[from] serde_v8::Error),
    #[error("js error: {0}")]
    JsError(#[from] Box<deno_core::error::JsError>),
    #[error("core error: {0}")]
    CoreError(#[from] deno_core::error::CoreError),
    #[error("dispatch task error: {0}")]
    DispatchTaskError(#[from] tokio::sync::mpsc::error::SendError<TypescriptTask>),
    #[error("result receive error: {0}")]
    OneshotReceiveError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("missing default export")]
    MissingDefaultExport,
}

pub struct ThreadTypescriptConfigRender {
    pub runtime: JsRuntime,
    pub transpile_options: TranspileOptions,
    pub transpile_module_options: TranspileModuleOptions,
    pub emit_options: EmitOptions,
}

impl ThreadTypescriptConfigRender {
    fn new() -> Self {
        // let deno_fetch = deno_fetch::deno_fetch::lazy_init();
        // let extensions = vec![deno_fetch];
        let runtime = JsRuntime::new(RuntimeOptions {
            // extensions,
            ..Default::default()
        });

        let transpile_options = TranspileOptions {
            ..Default::default()
        };

        let transpile_module_options = TranspileModuleOptions {
            ..Default::default()
        };

        let emit_options = EmitOptions {
            ..Default::default()
        };

        Self {
            runtime,
            transpile_options,
            transpile_module_options,
            emit_options,
        }
    }

    fn transpile_ts(&self, code: &str) -> Result<EmittedSourceText, TypescriptError> {
        let specifier = deno_core::ModuleSpecifier::parse("file:///config.ts")
            .expect("should be valid module specifier");
        let parsed = deno_ast::parse_module(ParseParams {
            specifier: specifier.clone(),
            text: code.into(),
            media_type: MediaType::TypeScript,
            capture_tokens: false,
            scope_analysis: false,
            maybe_syntax: None,
        })?;
        let transpiled = parsed.transpile(
            &self.transpile_options,
            &self.transpile_module_options,
            &self.emit_options,
        )?;
        Ok(transpiled.into_source())
    }

    async fn render_ts_config(&mut self, script: &str) -> Result<String, TypescriptError> {
        let transpiled = self.transpile_ts(script)?;
        let specifier = deno_core::ModuleSpecifier::parse("file:///config.js")
            .expect("should be valid module specifier");
        let main_module_id = self
            .runtime
            .load_main_es_module_from_code(&specifier, transpiled.text)
            .await?;
        self.runtime.mod_evaluate(main_module_id).await?;
        self.runtime
            .run_event_loop(PollEventLoopOptions::default())
            .await?;
        let json_value: serde_json::Value = {
            let main_module_namespace = self.runtime.get_module_namespace(main_module_id)?;
            deno_core::scope!(scope, self.runtime);
            let local = deno_core::v8::Local::new(scope, main_module_namespace);
            let default_export = local
                .get(
                    scope,
                    deno_core::v8::String::new(scope, "default")
                        .expect("default should be valid scope")
                        .into(),
                )
                .ok_or_else(|| TypescriptError::MissingDefaultExport)?;
            serde_v8::from_v8(scope, default_export)?
        };
        Ok(json_value.to_string())
    }
}

pub struct TypescriptConfigRender {
    pub task_sender: tokio::sync::mpsc::UnboundedSender<TypescriptTask>,
    pub task_handle: std::thread::JoinHandle<()>,
}

pub struct TypescriptTask {
    pub script: String,
    pub result_sender: tokio::sync::oneshot::Sender<Result<String, TypescriptError>>,
}

impl TypescriptConfigRender {
    pub fn spawn() -> Self {
        let (task_sender, mut task_receiver) =
            tokio::sync::mpsc::unbounded_channel::<TypescriptTask>();
        let thread = std::thread::Builder::new().name("sbk-config-render".to_string()).spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("should build typescript renderer runtime");
            rt.block_on(async move {
                let mut thread_runner = ThreadTypescriptConfigRender::new();
                loop {
                    match task_receiver.recv().await {
                        Some(task) => {
                            let result = thread_runner.render_ts_config(&task.script).await;
                            task.result_sender
                                .send(result)
                                .expect("shouldn't have another thread to set the result");
                        }
                        None => {
                            tracing::debug!(
                                "sender in main thread has been dropped, shutting down typescript renderer task"
                            );
                            break;
                        }
                    }
                }
            });
        }).expect("fail to spawn sbk-config-render thread");
        Self {
            task_sender,
            task_handle: thread,
        }
    }
    pub fn shutdown(self) {
        drop(self.task_sender);
        self.task_handle
            .join()
            .expect("should shutdown typescript renderer thread cleanly");
    }
    async fn render_config_inner(&self, script: &str) -> Result<String, TypescriptError> {
        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
        let task = TypescriptTask {
            script: script.to_string(),
            result_sender,
        };
        self.task_sender.send(task)?;
        let result = result_receiver.await??;
        Ok(result)
    }
}

impl super::ConfigRender for TypescriptConfigRender {
    async fn render_config(&self, script: &str) -> Result<String, super::ConfigError> {
        let result = self
            .render_config_inner(script)
            .await
            .map_err(|e| super::ConfigError::TypescriptError(e))?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "multi_thread")]
    async fn test_typescript_config_render() {
        let renderer = TypescriptConfigRender::spawn();
        let script = r#"
        function generateConfig() {
            return {
                "name": "example",
                "version": "1.0.0"
            };
        }
        // fetch resource from network
        export default generateConfig();
        "#;
        let result = renderer.render_config_inner(script).await.unwrap();
        println!("Rendered config: {}", result);
        assert_eq!(result, r#"{"name":"example","version":"1.0.0"}"#);
        renderer.shutdown();
    }
}
