use types::item::errors::ServerError;
use types::item::task_result::TaskFailure;

use crate::commands::errors::{CompilationError, VerificationError};

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
            Self::CompilationError(CompilationError::NothingToCompileError) => TaskFailure {
                error_type: ServerError::NothingToCompile,
                message: "Nothing to compile".to_string(),
            },
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

#[derive(thiserror::Error, Debug)]
pub enum VerifyProcessorError {
    #[error("Unsupported version: {0}")]
    VersionNotSupportedError(String),
    #[error("CompilationError: {0}")]
    VerificationError(#[from] VerificationError),
    #[error("UnknownError: {0}")]
    UnknownError(#[from] anyhow::Error),
}

impl Into<TaskFailure> for VerifyProcessorError {
    fn into(self) -> TaskFailure {
        match self {
            Self::VerificationError(err) => err.into(),
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
