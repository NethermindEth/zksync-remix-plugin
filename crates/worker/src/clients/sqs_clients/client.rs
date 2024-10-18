use aws_sdk_sqs::operation::delete_message::DeleteMessageOutput;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageOutput;
use aws_sdk_sqs::Client;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::clients::errors::{SqsDeleteError, SqsReceiveError};
use crate::clients::retriable::{handle_action_result, match_result, ActionHandler};

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

#[derive(Default)]
pub enum Action {
    #[default]
    Default, // TODO: get rid of this. crutches
    Receive(oneshot::Sender<Result<ReceiveMessageOutput, SqsReceiveError>>),
    Delete {
        receipt_handle: String,
        sender: oneshot::Sender<Result<DeleteMessageOutput, SqsDeleteError>>,
    },
}

impl ActionHandler for SqsClient {
    type Action = Action;
    async fn handle(&self, action: Action, state: Arc<AtomicU8>) -> Option<Self::Action> {
        match action {
            Action::Default => unreachable!(),
            Action::Receive(sender) => {
                let result = self.receive_attempt().await;
                handle_action_result(result, sender, state).map(|sender| Action::Receive(sender))
            }
            Action::Delete {
                receipt_handle,
                sender,
            } => {
                let result = self.delete_attempt(receipt_handle.clone()).await;
                handle_action_result(result, sender, state).map(|sender| Action::Delete {
                    sender,
                    receipt_handle,
                })
            }
        }
    }
}
