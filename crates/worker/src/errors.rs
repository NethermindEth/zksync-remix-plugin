use types::item::errors::ItemError;

use crate::clients::errors::{DBError, S3Error};

#[derive(thiserror::Error, Debug)]
pub enum PurgeError {
    #[error("DBError: {0}")]
    DBError(#[from] DBError),
    #[error("S3Error: {0}")]
    S3Error(#[from] S3Error),
    #[error("ItemError: {0}")]
    ItemError(#[from] ItemError),
}
