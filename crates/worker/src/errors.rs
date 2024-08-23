use aws_sdk_dynamodb::config::http::HttpResponse;
use aws_sdk_sqs::error::SdkError;
use aws_sdk_sqs::operation::delete_message::DeleteMessageError;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageError;

pub type ReceiveError = SdkError<ReceiveMessageError, HttpResponse>;
pub type DeleteError = SdkError<DeleteMessageError, HttpResponse>;
