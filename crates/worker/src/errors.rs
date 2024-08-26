use aws_sdk_dynamodb::config::http::HttpResponse;
use aws_sdk_dynamodb::operation::delete_item::DeleteItemError;
use aws_sdk_dynamodb::operation::get_item::GetItemError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_sqs::error::SdkError;
use aws_sdk_sqs::operation::delete_message::DeleteMessageError;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageError;
use types::{ItemError, Status};

// SQS related errors
pub(crate) type SqsReceiveError = SdkError<ReceiveMessageError, HttpResponse>;
pub(crate) type SqsDeleteError = SdkError<DeleteMessageError, HttpResponse>;

// DynamoDB related errors
pub(crate) type DBDeleteError = SdkError<DeleteItemError, HttpResponse>;
pub(crate) type DBGetError = SdkError<GetItemError, HttpResponse>;

pub(crate) type DBUpdateError = SdkError<UpdateItemError, HttpResponse>;

// S3 related errors
pub(crate) type S3ListObjectsError = SdkError<ListObjectsV2Error, HttpResponse>;
pub(crate) type S3GetObjectError = SdkError<GetObjectError, HttpResponse>;

#[derive(thiserror::Error, Debug)]
pub enum DBError {
    #[error(transparent)]
    DeleteItemError(#[from] DBDeleteError),
    #[error(transparent)]
    GetItemError(#[from] DBGetError),
    #[error(transparent)]
    ItemFormatError(#[from] ItemError),
    #[error(transparent)]
    UpdateItemError(#[from] UpdateItemError)
}

#[derive(thiserror::Error, Debug)]
pub enum S3Error {
    #[error("Invalid object")]
    InvalidObjectError,
    #[error(transparent)]
    GetObjectError(#[from] S3GetObjectError),
    #[error(transparent)]
    ListObjectsError(#[from] S3ListObjectsError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ByteStreamError(#[from] aws_smithy_types::byte_stream::error::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum CompilationError {
    #[error(transparent)]
    DBError(#[from] DBError),
    #[error(transparent)]
    S3Error(#[from] S3Error),
    #[error("Item isn't id DB: {0}")]
    NoDBItemError(String),
    #[error("Unexpected status: {0}")]
    UnexpectedStatusError(String), // ignorable
    #[error("Unsupported version: {0}")]
    VersionNotSupported(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DBError(#[from] DBError),
}
