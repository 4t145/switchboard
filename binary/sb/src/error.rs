use tokio::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    
    #[error("TOML error")]
    Toml(#[from] toml_edit::TomlError),
    
    #[error("Unimplemented")]
    Unimplemented
}

pub type Result<T> = ::core::result::Result<T, Error>;
