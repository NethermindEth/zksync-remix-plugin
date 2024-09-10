use crate::errors::{DBError, S3Error};

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to compile: {0}")]
    CompilationFailureError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum PreparationError {
    #[error("DBError: {0}")]
    DBError(#[from] DBError),
    #[error("S3Error: {0}")]
    S3Error(#[from] S3Error),
    #[error("Item isn't id DB: {0}")]
    NoDBItemError(String),
    #[error("Unexpected status: {0}")]
    UnexpectedStatusError(String),
    #[error("Unsupported version: {0}")]
    VersionNotSupported(String),
}

// impl CompilationError {
//     pub fn recoverable(&self) -> bool {
//         match self {
//             CompilationError::DBError(_) | CompilationError::S3Error(_) => true,
//             CompilationError::NoDBItemError(_)
//             | CompilationError::UnexpectedStatusError(_)
//             | CompilationError::IoError(_)
//             | CompilationError::VersionNotSupported(_)
//             | CompilationError::CompilationFailureError(_) => false,
//         }
//     }
// }
