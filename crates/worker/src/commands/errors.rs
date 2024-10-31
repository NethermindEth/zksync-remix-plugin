use types::item::errors::ServerError;
use types::item::task_result::TaskFailure;

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error("CompilationFailureError: {0}")]
    CompilationFailureError(String),
    #[error("Nothing to compile")]
    NothingToCompileError,
    #[error("UnknownError: {0}")]
    UnknownError(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum VerificationError {
    #[error("UnknownNetwork: {0}")]
    UnknownNetworkError(String),
    #[error("VerificationFailureError : {0}")]
    VerificationFailureError(String),
    #[error("UnknownError: {0}")]
    UnknownError(#[from] anyhow::Error),
}

impl Into<TaskFailure> for &VerificationError {
    fn into(self) -> TaskFailure {
        match self {
            VerificationError::UnknownNetworkError(err) => TaskFailure {
                error_type: ServerError::VerificationError,
                message: err.to_string(),
            },
            VerificationError::VerificationFailureError(err) => TaskFailure {
                error_type: ServerError::UnsupportedCompilerVersion,
                message: err.clone(),
            },
            VerificationError::UnknownError(err) => TaskFailure {
                error_type: ServerError::InternalError,
                message: err.to_string(),
            },
        }
    }
}

impl Into<TaskFailure> for VerificationError {
    fn into(self) -> TaskFailure {
        (&self).into()
    }
}
