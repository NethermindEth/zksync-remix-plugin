use types::item::errors::ServerError;
use types::item::task_result::TaskFailure;

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

impl Into<TaskFailure> for CompileProcessorError {
    fn into(self) -> TaskFailure {
        match self {
            Self::CompilationError(err) => TaskFailure {
                error_type: ServerError::CompilationError,
                message: err.to_string(),
            },
            Self::VersionNotSupportedError(err) => TaskFailure {
                error_type: ServerError::UnsupportedCompilerVersion,
                message: err,
            },
            Self::UnknownError(err) => TaskFailure {
                error_type: ServerError::InternalError,
                message: err.to_string(),
            },
        }
    }
}
