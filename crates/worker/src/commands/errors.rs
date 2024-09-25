use crate::clients::errors::{DBError, S3Error};

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error("Failed to compile: {0}")]
    CompilationFailureError(String),
    #[error("UnknownError: {0}")]
    UnknownError(#[from] anyhow::Error),
}
