use crate::commands::errors::CompilationError;

#[derive(thiserror::Error, Debug)]
pub enum CompileProcessorError {
    #[error("Unsupported version: {0}")]
    VersionNotSupportedError(String),
    #[error("CompilationError: {0}")]
    CompilationError(#[from] CompilationError),
    #[error("UnknownError: {0}")]
    UnknownError(#[from] anyhow::Error),
}
