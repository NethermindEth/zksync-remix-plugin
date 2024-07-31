use std::io::Error as IoError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Failed to execute command: {0}")]
    FailedToExecuteCommand(IoError),
    #[error("Failed to read output: {0}")]
    FailedToReadOutput(IoError),
    #[error("UTF8 error: {0}")]
    UTF8Error(#[from] std::string::FromUtf8Error),
    #[error("Failed to read dir: {0}")]
    FailedToReadDir(IoError),
    #[error("Failed to read file: {0}")]
    FailedToReadFile(IoError),
    #[error("Failed to parse string")]
    FailedToParseString,
    #[error("File extension <{0}> not supported")]
    FileExtensionNotSupported(String),
    #[error("Cairo version {0} not found")]
    CairoVersionNotFound(String),
    #[error("Failed to save file: {0}")]
    FailedToSaveFile(IoError),
    #[error("Failed to read filename")]
    FailedToReadFilename,
    #[error("Task queue is full")]
    QueueIsFull,
    #[error("Rate limiter is not in the Rocket state")]
    RateLimiterNotInState,
    #[error("Failed to fetch client IP from the request")]
    FailedToGetClientIp,
    #[error("Too many requests")]
    TooManyRequests,
    #[error("Error while trying to unlock mutex")]
    MutexUnlockError,
    #[error("Error('s) raised while trying to parse sol file: \n{0}")]
    FailedToParseSol(String),
    #[error("Failed to write file: {0}")]
    FailedToWriteFile(IoError),
    #[error("Unsupported version: {0}")]
    VersionNotSupported(String),
    #[error("Failed to get parent directory")]
    FailedToGetParentDir,
    #[error("Failed to remove directory")]
    FailedToRemoveDir,
    #[error("Failed to remove file")]
    FailedToRemoveFile(IoError),
    #[error("Unknown network selected: {0}")]
    UnknownNetwork(String),
}

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error(transparent)]
    RocketError(#[from] rocket::Error),
    #[error(transparent)]
    PrometheusError(#[from] prometheus::Error),
    #[error(transparent)]
    LoggingGlobalError(#[from] tracing::dispatcher::SetGlobalDefaultError),
}

pub type Result<T, E = ApiError> = std::result::Result<T, E>;
