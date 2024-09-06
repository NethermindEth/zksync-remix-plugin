use std::num::NonZeroUsize;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, warn};
use types::SqsMessage;

use crate::commands::compile::compile;
use crate::dynamodb_client::DynamoDBClient;
use crate::purgatory::{Purgatory, State};
use crate::s3_client::S3Client;
use crate::sqs_client::wrapper::SqsClientWrapper;
use crate::sqs_listener::{SqsListener, SqsReceiver};

pub struct EngineBuilder {
    sqs_client: SqsClientWrapper,
    db_client: DynamoDBClient,
    s3_client: S3Client,
    state: State,
    running_workers: Vec<RunningEngine>,
}

impl EngineBuilder {
    pub fn new(
        sqs_client: SqsClientWrapper,
        db_client: DynamoDBClient,
        s3_client: S3Client,
        state: State,
    ) -> Self {
        EngineBuilder {
            sqs_client,
            db_client,
            s3_client,
            state,
            running_workers: vec![],
        }
    }

    pub fn start(self, num_workers: NonZeroUsize) -> RunningEngine {
        let sqs_listener = SqsListener::new(self.sqs_client, Duration::from_millis(500));

        RunningEngine::new(
            sqs_listener,
            self.db_client,
            self.s3_client,
            self.state,
            num_workers.get(),
        )
    }
}

pub struct RunningEngine {
    sqs_listener: SqsListener,
    purgatory: Purgatory,
    num_workers: usize,
    worker_threads: Vec<JoinHandle<()>>,
}

impl RunningEngine {
    pub fn new(
        sqs_listener: SqsListener,
        db_client: DynamoDBClient,
        s3_client: S3Client,
        state: State,
        num_workers: usize,
    ) -> Self {
        let purgatory = Purgatory::new(state);

        let mut worker_threads = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            // Start worker
            let sqs_receiver = sqs_listener.receiver();
            let db_client_copy = db_client.clone();
            let s3_client_copy = s3_client.clone();
            let purgatory_copy = purgatory.clone();

            worker_threads.push(tokio::spawn(async move {
                RunningEngine::worker(sqs_receiver, db_client_copy, s3_client_copy, purgatory_copy)
                    .await;
            }));
        }

        Self {
            sqs_listener,
            purgatory,
            num_workers,
            worker_threads,
        }
    }

    async fn worker(
        sqs_receiver: SqsReceiver,
        db_client: DynamoDBClient,
        s3_client: S3Client,
        mut purgatory: Purgatory,
    ) {
        // TODO: process error
        while let Ok(message) = sqs_receiver.recv().await {
            let receipt_handle = if let Some(receipt_handle) = message.receipt_handle {
                receipt_handle
            } else {
                continue;
            };

            let body = if let Some(body) = message.body {
                body
            } else {
                warn!("Has handle but not body");
                if let Err(err) = sqs_receiver.delete_message(receipt_handle).await {
                    warn!("{}", err);
                }

                continue;
            };

            let sqs_message = match serde_json::from_str::<SqsMessage>(&body) {
                Ok(sqs_message) => sqs_message,
                Err(err) => {
                    error!("Could not deserialize message: {}", err.to_string());
                    if let Err(err) = sqs_receiver.delete_message(receipt_handle).await {
                        warn!("{}", err);
                    }

                    continue;
                }
            };

            purgatory.add_task(&sqs_message).await;
            match sqs_message {
                SqsMessage::Compile { request } => {
                    let result = compile(request, &db_client, &s3_client).await; // TODO:
                    match result {
                        Ok(()) => purgatory.update_task().await,
                        Err(err) => {
                            if err.recoverable() {
                                warn!("recoverable error after compilation: {}", err);
                                continue;
                            } else {
                                // delete from SQS
                                warn!("unrecoverable error after compilation: {}", err);
                                purgatory.update_task().await;
                            }
                        }
                    }
                }
                SqsMessage::Verify { request } => {} // TODO;
            }

            if let Err(err) = sqs_receiver.delete_message(receipt_handle).await {
                warn!("{}", err);
            }
        }
    }

    pub async fn wait(self) {
        futures::future::join_all(self.worker_threads).await;
    }
}

// what are we purging?
