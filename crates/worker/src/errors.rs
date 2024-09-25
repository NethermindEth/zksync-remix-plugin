use types::item::ItemError;

use crate::clients::errors::{DBError, S3Error};
use crate::commands::errors::{CompilationError};

#[derive(thiserror::Error, Debug)]
pub enum PurgeError {
    #[error("DBError: {0}")]
    DBError(#[from] DBError),
    #[error("S3Error: {0}")]
    S3Error(#[from] S3Error),
    #[error("ItemError: {0}")]
    ItemError(#[from] ItemError),
}
#[derive(thiserror::Error, Debug)]
pub enum CompileProcessorError {
    #[error("Unsupported version: {0}")]
    VersionNotSupportedError(String),
    #[error("CompilationError: {0}")]
    CompilationError(#[from] CompilationError),
    #[error("UnknownError: {0}")]
    UnknownError(#[from] anyhow::Error),
}
