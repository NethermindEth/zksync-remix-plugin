use types::item::ItemError;

use crate::clients::errors::DBError;

#[derive(thiserror::Error, Debug)]
pub enum GlobalPurgeError {
    #[error("DBError: {0}")]
    DBError(#[from] DBError),
    #[error("ItemError: {0}")]
    ItemError(#[from] ItemError),
}
