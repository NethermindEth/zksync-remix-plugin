use types::item::ItemError;

use crate::clients::errors::{DBError, S3Error, SqsDeleteError};
use crate::commands::errors::{CommandResultHandleError, PreparationError};

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
pub enum MessageProcessorError {
    #[error("PreparationError: {0}")]
    PreparationError(#[from] PreparationError),
    #[error("CommandResultHandleError: {0}")]
    CommandResultHandleError(#[from] CommandResultHandleError),
    #[error("S3Error: {0}")]
    S3Error(#[from] S3Error),
    #[error("SqsDeleteError: {0}")]
    SqsDeleteError(#[from] SqsDeleteError),
}
