use crate::errors::{SqsDeleteError, SqsReceiveError};
use crate::sqs_client::SqsClient;
use aws_sdk_sqs::operation::delete_message::DeleteMessageOutput;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageOutput;
use aws_sdk_sqs::Client;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Instant};

pub enum Action {
    Default,
    Receive(oneshot::Sender<Result<ReceiveMessageOutput, SqsReceiveError>>),
    Delete {
        receipt_handle: String,
        sender: oneshot::Sender<Result<DeleteMessageOutput, SqsDeleteError>>,
    },
}

impl Default for Action {
    fn default() -> Self {
        Action::Default
    }
}

enum State {
    Connected = 0,
    Reconnecting = 1,
}

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

        tokio::spawn(Self::worker(client.clone(), state.clone(), receiver));

        Self {
            client,
            actions_sender: sender,
            state,
        }
    }

    // TODO: start
    async fn worker(client: SqsClient, state: Arc<AtomicU8>, mut receiver: mpsc::Receiver<Action>) {
        const SLEEP_DURATION: Duration = Duration::from_secs(3);
        let mut pending_actions = vec![];

        loop {
            if pending_actions.is_empty() {
                if let Some(action) = receiver.recv().await {
                    pending_actions.push(action);
                } else {
                    return;
                }
            }

            Self::resend_pending_actions(&mut pending_actions, &client, &state).await;

            let start_time = Instant::now();
            let value = select! {
                value = receiver.recv() => value,
                _ = sleep(SLEEP_DURATION) => continue,
            };

            if let Some(action) = value {
                pending_actions.push(action);
            } else {
                return;
            }

            let elapsed = start_time.elapsed();
            if let Some(remaining_sleep) = SLEEP_DURATION.checked_sub(elapsed) {
                sleep(remaining_sleep).await;
            }
        }
    }

    pub async fn resend_pending_actions(
        pending_actions: &mut Vec<Action>,
        client: &SqsClient,
        state: &Arc<AtomicU8>,
    ) {
        let mut pivot = 0;
        for i in 0..pending_actions.len() {
            let action = std::mem::take(&mut pending_actions[i]);
            match action {
                Action::Receive(sender) => match client.receive_attempt().await {
                    Ok(Some(val)) => {
                        state.store(State::Connected as u8, Ordering::Release);
                        let _ = sender.send(Ok(val));
                    }
                    Err(err) => {
                        let _ = sender.send(Err(err));
                    }
                    Ok(None) => {
                        // Keeping in the array to resend.
                        pending_actions[pivot] = Action::Receive(sender);
                        pivot += 1;
                    }
                },
                Action::Delete {
                    receipt_handle,
                    sender,
                } => match client.delete_attempt(receipt_handle.clone()).await {
                    Ok(Some(val)) => {
                        state.store(State::Connected as u8, Ordering::Release);
                        let _ = sender.send(Ok(val));
                    }
                    Err(err) => {
                        let _ = sender.send(Err(err));
                    }
                    Ok(None) => {
                        pending_actions[pivot] = Action::Delete {
                            receipt_handle,
                            sender,
                        };
                        pivot += 1;
                    }
                },
                Action::Default => unreachable!(),
            };
        }

        pending_actions.truncate(pivot);
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
