use crate::clients::errors::{SqsDeleteError, SqsReceiveError};
use aws_sdk_sqs::operation::delete_message::DeleteMessageOutput;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageOutput;
use aws_sdk_sqs::Client;

macro_rules! match_result {
    ($err_type:ident, $result:expr) => {
        match $result {
            Ok(val) => Ok(Some(val)),
            Err(err) => match err {
                $err_type::ConstructionFailure(_) => Err(err),
                $err_type::TimeoutError(_) => Ok(None),
                $err_type::DispatchFailure(dispatch_err) => {
                    if dispatch_err.is_io() {
                        return Ok(None);
                    }
                    if dispatch_err.is_timeout() {
                        return Ok(None);
                    }
                    if dispatch_err.is_user() {
                        return Err($err_type::DispatchFailure(dispatch_err));
                    }
                    if let Some(other) = dispatch_err.as_other() {
                        return match other {
                            aws_config::retry::ErrorKind::ClientError => {
                                Err($err_type::DispatchFailure(dispatch_err))
                            }
                            _ => Ok(None),
                        };
                    }
                    Err($err_type::DispatchFailure(dispatch_err))
                }
                other => Err(other),
            },
        }
    };
}

#[derive(Clone)]
pub struct SqsClient {
    pub client: Client,
    pub queue_url: String,
}

impl SqsClient {
    pub fn new(client: Client, queue_url: impl Into<String>) -> Self {
        Self {
            client,
            queue_url: queue_url.into(),
        }
    }

    pub async fn receive_attempt(&self) -> Result<Option<ReceiveMessageOutput>, SqsReceiveError> {
        let result = self
            .client
            .receive_message()
            .queue_url(self.queue_url.clone())
            .max_number_of_messages(1)
            .send()
            .await;

        match_result!(SqsReceiveError, result)
    }

    pub async fn delete_attempt(
        &self,
        receipt_handle: impl Into<String>,
    ) -> Result<Option<DeleteMessageOutput>, SqsDeleteError> {
        let result = self
            .client
            .delete_message()
            .queue_url(self.queue_url.clone())
            .receipt_handle(receipt_handle)
            .send()
            .await;

        match_result!(SqsDeleteError, result)
    }
}
