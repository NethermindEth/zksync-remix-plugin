use crate::clients::errors::{DBError, S3Error};

#[derive(thiserror::Error, Debug)]
pub enum PreparationError {
    #[error("Unsupported version: {0}")]
    VersionNotSupportedError(String),
    #[error("UnknownError: {0}")]
    UnknownError(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error("Failed to compile: {0}")]
    CompilationFailureError(String),
    #[error("UnknownError: {0}")]
    UnknownError(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum CommandResultHandleError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("DBError: {0}")]
    DBError(#[from] DBError),
    #[error("S3Error: {0}")]
    S3Error(#[from] S3Error),
}
