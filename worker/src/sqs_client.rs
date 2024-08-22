use crate::errors::{DeleteError, ReceiveError};
use aws_config::retry::ErrorKind;
use aws_sdk_sqs::operation::delete_message::DeleteMessageOutput;
use aws_sdk_sqs::operation::receive_message::ReceiveMessageOutput;
use aws_sdk_sqs::Client;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Mutex};
use tokio::time::sleep;

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
                            ErrorKind::ClientError => Err($err_type::DispatchFailure(dispatch_err)),
                            _ => Ok(None),
                        };
                    }
                    Err($err_type::DispatchFailure(dispatch_err))
                }
                other => Err(other),
            },
            Err(err) => Err(err.into()),
        }
    };
}

enum Action {
    Default,
    Receive(oneshot::Sender<Result<ReceiveMessageOutput, ReceiveError>>),
    Delete {
        receipt_handle: String,
        sender: oneshot::Sender<Result<DeleteMessageOutput, DeleteError>>,
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
pub struct SqsClient {
    client: Client,
    queue_url: String,
    pending_actions: Arc<Mutex<Vec<Action>>>,
    state: Arc<AtomicU8>,
}

impl SqsClient {
    pub fn new(client: Client, queue_url: impl Into<String>) -> Self {
        Self {
            client,
            queue_url: queue_url.into(),
            pending_actions: Arc::new(Mutex::new(vec![])),
            state: Arc::new(AtomicU8::new(State::Connected as u8)),
        }
    }

    async fn receive_attempt(&self) -> Result<Option<ReceiveMessageOutput>, ReceiveError> {
        let result = self
            .client
            .receive_message()
            .queue_url(self.queue_url.clone())
            .max_number_of_messages(1)
            .send()
            .await;

        match_result!(ReceiveError, result)
    }

    async fn delete_attempt(
        &self,
        receipt_handle: impl Into<String>,
    ) -> Result<Option<DeleteMessageOutput>, DeleteError> {
        let result = self
            .client
            .delete_message()
            .queue_url(self.queue_url.clone())
            .receipt_handle(receipt_handle)
            .send()
            .await;

        match_result!(DeleteError, result)
    }

    async fn worker(self) {
        loop {
            let mut actions = self.pending_actions.lock().await;

            let mut pivot = 0;
            for i in 0..actions.len() {
                let action = std::mem::take(&mut actions[i]);
                match action {
                    Action::Receive(sender) => match self.receive_attempt().await {
                        Ok(Some(val)) => {
                            self.state.store(State::Connected as u8, Ordering::Release);
                            if sender.send(Ok(val)).is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            if sender.send(Err(err)).is_err() {
                                break;
                            }
                        }
                        Ok(None) => {
                            // Keeping in the array to resend.
                            actions[pivot] = Action::Receive(sender);
                            pivot += 1;
                        }
                    },
                    Action::Delete {
                        receipt_handle,
                        sender,
                    } => match self.delete_attempt(receipt_handle.clone()).await {
                        Ok(Some(val)) => {
                            self.state.store(State::Connected as u8, Ordering::Release);
                            if sender.send(Ok(val)).is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            if sender.send(Err(err)).is_err() {
                                break;
                            }
                        }
                        Ok(None) => {
                            actions[pivot] = Action::Delete {
                                receipt_handle,
                                sender,
                            };
                            pivot += 1;
                        }
                    },
                    Action::Default => unreachable!(),
                };
            }

            actions.truncate(pivot);
            drop(actions);

            sleep(Duration::from_secs(3)).await;
        }
    }

    pub async fn receive_message(&self) -> Result<ReceiveMessageOutput, ReceiveError> {
        match self.state.load(Ordering::Acquire) {
            0 => match self.receive_attempt().await {
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
        self.pending_actions
            .lock()
            .await
            .push(Action::Receive(sender));

        receiver.await.unwrap() // TODO: for now
    }

    pub async fn delete_message(
        &self,
        receipt_handle: impl Into<String>,
    ) -> Result<(), DeleteError> {
        let receipt_handle = receipt_handle.into();
        match self.state.load(Ordering::Acquire) {
            0 => match self.delete_attempt(receipt_handle.clone()).await {
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Ok(Some(_)) => return Ok(()),
                Err(err) => return Err(err),
            },
            1 => {}
            _ => unreachable!(),
        };

        // State::Reconnecting branch
        let (sender, receiver) = oneshot::channel();
        self.pending_actions.lock().await.push(Action::Delete {
            receipt_handle,
            sender,
        });

        receiver.await.unwrap().map(|_| ()) // TODO: for now
    }
}
