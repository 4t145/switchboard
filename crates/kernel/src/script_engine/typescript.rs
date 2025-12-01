use deno_ast::EmitOptions;
use deno_ast::EmittedSourceText;
use deno_ast::ParseDiagnostic;
use deno_ast::TranspileError;
use deno_ast::TranspileModuleOptions;
use deno_ast::TranspileOptions;
use deno_ast::{MediaType, ParseParams};
use deno_core::JsRuntime;
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
    #[error("dispatch task error: {0}")]
    DispatchTaskError(#[from] tokio::sync::mpsc::error::SendError<TypescriptTask>),
    #[error("result receive error: {0}")]
    OneshotReceiveError(#[from] tokio::sync::oneshot::error::RecvError),
}

pub struct ThreadTypescriptConfigRender {
    pub runtime: JsRuntime,
    pub transpile_options: TranspileOptions,
    pub transpile_module_options: TranspileModuleOptions,
    pub emit_options: EmitOptions,
}

impl ThreadTypescriptConfigRender {
    fn new() -> Self {
        let runtime = JsRuntime::new(RuntimeOptions {
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
        let parsed = deno_ast::parse_module(ParseParams {
            specifier: deno_core::ModuleSpecifier::parse("file:///config.ts").unwrap(),
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

    fn render_ts_config(&mut self, script: &str) -> Result<String, TypescriptError> {
        let transpiled = self.transpile_ts(script)?;
        let result = self
            .runtime
            .execute_script("config_generation.ts", transpiled.text.to_owned())?;
        let json_value: serde_json::Value = {
            deno_core::scope!(scope, self.runtime);
            let local = deno_core::v8::Local::new(scope, result);
            serde_v8::from_v8(scope, local)?
        };
        Ok(json_value.to_string())
    }
}

pub struct TypescriptConfigRender {
    pub task_sender: tokio::sync::mpsc::UnboundedSender<TypescriptTask>,
    pub thread_handle: std::thread::JoinHandle<()>,
}

pub struct TypescriptTask {
    pub script: String,
    pub result_sender: tokio::sync::oneshot::Sender<Result<String, TypescriptError>>,
}
impl TypescriptConfigRender {
    pub fn spawn() -> Self {
        let (task_sender, mut task_receiver) =
            tokio::sync::mpsc::unbounded_channel::<TypescriptTask>();

        let thread = std::thread::spawn(move || {
            let mut thread_runner = ThreadTypescriptConfigRender::new();
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("fail to build tokio async runtime");
            rt.block_on(async move {
                loop {
                    match task_receiver.recv().await {
                        Some(task) => {
                            tokio::task::block_in_place(||  {
                                let result = thread_runner.render_ts_config(&task.script);
                                task.result_sender
                                .send(result)
                                .expect("shouldn't have another thread to set the result");
                            });
                        }
                        None => {
                            tracing::debug!("sender in main thread has been dropped, shutting down typescript renderer thread");
                            break;
                        }
                    }
                }
            });
        });
        Self {
            task_sender,
            thread_handle: thread,
        }
    }
    pub fn shutdown(self) {
        drop(self.task_sender);
        self.thread_handle
            .join()
            .expect("should shutdown typescript renderer thread cleanly");
    }
    pub async fn render_config(&self, script: &str) -> Result<String, TypescriptError> {
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
            .render_config(script)
            .await
            .map_err(|e| super::ConfigError::TypescriptError(e))?;
        Ok(result)
    }
}
