use crate::errors::{SqsDeleteError, SqsReceiveError};
use async_channel::{Receiver, Recv, Sender};
use aws_sdk_sqs::config::http::HttpResponse;
use aws_sdk_sqs::error::SdkError;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageError;
use aws_sdk_sqs::types::Message;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::sqs_client::wrapper::SqsClientWrapper;
use crate::sqs_client::SqsClient;

pub struct SqsListener {
    handle: JoinHandle<Result<(), SqsReceiveError>>,
    receiver: Receiver<Message>,
    client: SqsClientWrapper,
}

impl SqsListener {
    pub fn new(client: SqsClientWrapper, poll_interval: Duration) -> Self {
        // TODO: unbounded?
        let (sender, receiver) = async_channel::bounded(1000);
        let handle = tokio::spawn(Self::listen(client.clone(), sender, poll_interval));

        Self {
            handle,
            receiver,
            client,
        }
    }

    async fn listen(
        client: SqsClientWrapper,
        sender: Sender<Message>,
        poll_interval: Duration,
    ) -> Result<(), SdkError<ReceiveMessageError, HttpResponse>> {
        loop {
            let response = client.receive_message().await?;
            let messages = if let Some(messages) = response.messages {
                messages
            } else {
                continue;
            };

            for message in messages {
                if sender.send(message).await.is_err() {
                    return Ok(());
                }
            }

            sleep(poll_interval).await;
        }
    }

    pub fn receiver(&self) -> SqsReceiver {
        SqsReceiver {
            client: self.client.clone(),
            receiver: self.receiver.clone(),
        }
    }

    pub fn handle(self) -> JoinHandle<Result<(), SqsReceiveError>> {
        self.handle
    }
}

pub struct SqsReceiver {
    client: SqsClientWrapper,
    receiver: Receiver<Message>,
}

impl SqsReceiver {
    pub fn recv(&self) -> Recv<'_, Message> {
        self.receiver.recv()
    }

    pub async fn delete_message(
        &self,
        receipt_handle: impl Into<String>,
    ) -> Result<(), SqsDeleteError> {
        self.client.delete_message(receipt_handle).await
    }
}
