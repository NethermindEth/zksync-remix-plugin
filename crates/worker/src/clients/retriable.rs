use crate::clients::errors::S3DeleteObjectError;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Instant};

macro_rules! match_result {
    ($err_type:ident, $result:expr) => {
        match $result {
            Ok(val) => Ok(Some(val)),
            Err(err) => match err {
                $err_type::ConstructionFailure(_) => Err(err),
                $err_type::TimeoutError(_) => Ok(None),
                $err_type::DispatchFailure(dispatch_err) => {
                    if dispatch_err.is_io() {
                        Ok(None)
                    } else if dispatch_err.is_timeout() {
                        Ok(None)
                    } else if dispatch_err.is_user() {
                        Err($err_type::DispatchFailure(dispatch_err))
                    } else if let Some(other) = dispatch_err.as_other() {
                        match other {
                            aws_config::retry::ErrorKind::ClientError => {
                                Err($err_type::DispatchFailure(dispatch_err))
                            }
                            _ => Ok(None),
                        }
                    } else {
                        Err($err_type::DispatchFailure(dispatch_err))
                    }
                }
                other => Err(other),
            },
        }
    };
}
pub(crate) use match_result;

pub trait ActionHandler {
    type Action: Default;
    async fn handle(&self, action: Self::Action, state: Arc<AtomicU8>) -> Option<Self::Action>;
}

pub(crate) fn handle_action_result<T, E>(
    result: Result<Option<T>, E>,
    sender: oneshot::Sender<Result<T, E>>,
    state: Arc<AtomicU8>,
) -> Option<oneshot::Sender<Result<T, E>>>
where
    T: Send + 'static,
    E: Send + 'static,
{
    match result {
        Ok(Some(val)) => {
            state.store(State::Connected as u8, Ordering::Release);
            let _ = sender.send(Ok(val));
            None
        }
        Err(err) => {
            let _ = sender.send(Err(err));
            None
        }
        Ok(None) => {
            state.store(State::Reconnecting as u8, Ordering::Release);
            Some(sender)
        }
    }
}
pub enum State {
    Connected = 0,
    Reconnecting = 1,
}

pub struct Retrier<T: ActionHandler + Clone> {
    client: T,
    receiver: mpsc::Receiver<T::Action>,
    state: Arc<AtomicU8>,
}

impl<T: ActionHandler + Clone> Retrier<T> {
    pub fn new(client: T, receiver: mpsc::Receiver<T::Action>, state: Arc<AtomicU8>) -> Self {
        Self {
            client,
            receiver,
            state,
        }
    }

    pub async fn start(mut self) {
        const SLEEP_DURATION: Duration = Duration::from_secs(3);
        // add lru instead
        let mut pending_actions = vec![];

        loop {
            if pending_actions.is_empty() {
                if let Some(action) = self.receiver.recv().await {
                    pending_actions.push(action);
                } else {
                    return;
                }
            }

            self.resend_pending_actions(&mut pending_actions).await;

            let start_time = Instant::now();
            let value = select! {
                value = self.receiver.recv() => value,
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

    pub async fn resend_pending_actions(&self, pending_actions: &mut Vec<T::Action>) {
        let mut pivot = 0;
        for i in 0..pending_actions.len() {
            let action = std::mem::take(&mut pending_actions[i]);
            let action_unhandled = self.client.handle(action, self.state.clone()).await;

            // Keeping in the array to resend.
            if let Some(action) = action_unhandled {
                pending_actions[pivot] = action;
                pivot += 1;
            }
        }

        pending_actions.truncate(pivot);
    }
}
