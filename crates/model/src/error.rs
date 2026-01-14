use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]

pub struct ErrorStack {
    pub frames: Vec<ErrorStackFrame>,
}

impl std::fmt::Display for ErrorStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Error stack trace:")?;
        for frame in &self.frames {
            writeln!(f, "{}: {}", frame.type_name, frame.error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ErrorStack {}

#[derive(
    Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]

pub struct ErrorStackFrame {
    pub error: String,
    pub type_name: String,
}

impl ErrorStack {
    pub fn from_std<E: std::error::Error + 'static>(e: E) -> Self {
        let mut frames = Vec::new();
        let mut current: Option<&(dyn std::error::Error + 'static)> = Some(&e);
        while let Some(err) = current {
            frames.push(ErrorStackFrame {
                error: err.to_string(),
                type_name: std::any::type_name_of_val(err).to_owned(),
            });
            current = err.source();
        }
        Self { frames }
    }
}

#[derive(
    Debug, Clone, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode, PartialEq, Eq,
)]

pub enum ResultObject<T> {
    Data(T),
    Error(ErrorStack),
}

impl<T, E> From<Result<T, E>> for ResultObject<T>
where
    E: std::error::Error + 'static,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => ResultObject::Data(data),
            Err(e) => ResultObject::Error(ErrorStack::from_std(e)),
        }
    }
}
