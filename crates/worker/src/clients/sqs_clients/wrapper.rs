use aws_sdk_sqs::operation::delete_message::DeleteMessageOutput;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageOutput;
use aws_sdk_sqs::Client;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

use crate::clients::errors::{SqsDeleteError, SqsReceiveError};
use crate::clients::retriable::{Retrier, State};
use crate::clients::sqs_clients::client::{Action, SqsClient};

#[derive(Clone)]
pub struct SqsClientWrapper {
    client: SqsClient,
    actions_sender: mpsc::Sender<Action>,
    state: Arc<AtomicU8>,
}

impl SqsClientWrapper {
    pub fn new(client: Client, queue_url: impl Into<String>) -> Self {
        let client = SqsClient::new(client, queue_url);
        let state = Arc::new(AtomicU8::new(State::Connected as u8));
        let (sender, receiver) = mpsc::channel(1000);

        let retrier = Retrier::new(client.clone(), receiver, state.clone());
        tokio::spawn(retrier.start());

        Self {
            client,
            actions_sender: sender,
            state,
        }
    }

    pub async fn receive_message(&self) -> Result<ReceiveMessageOutput, SqsReceiveError> {
        match self.state.load(Ordering::Acquire) {
            0 => match self.client.receive_attempt().await {
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Ok(Some(val)) => return Ok(val),
                Err(err) => return Err(err),
            },
            1 => {}
            _ => unreachable!(),
        };

        // State::Reconnecting branch
        let (sender, receiver) = oneshot::channel();
        self.actions_sender.send(Action::Receive(sender)).await;
        receiver.await.unwrap() // TODO: for now
    }

    pub async fn delete_message(
        &self,
        receipt_handle: impl Into<String>,
    ) -> Result<DeleteMessageOutput, SqsDeleteError> {
        let receipt_handle = receipt_handle.into();
        match self.state.load(Ordering::Acquire) {
            0 => match self.client.delete_attempt(receipt_handle.clone()).await {
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Ok(Some(value)) => return Ok(value),
                Err(err) => return Err(err),
            },
            1 => {}
            _ => unreachable!(),
        };

        // State::Reconnecting branch
        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(Action::Delete {
                receipt_handle,
                sender,
            })
            .await;

        receiver.await.unwrap() // TODO: for now
    }
}
