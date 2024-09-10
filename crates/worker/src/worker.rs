use std::num::NonZeroUsize;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use types::SqsMessage;

use crate::clients::dynamodb_client::DynamoDBClient;
use crate::clients::s3_client::S3Client;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::commands::compile::do_compile;
use crate::commands::errors::PreparationError;
use crate::commands::utils::{
    on_compilation_failed, on_compilation_success, prepare_compile_input,
};
use crate::purgatory::{Purgatory, State};
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
        let purgatory = Purgatory::new(state, db_client.clone(), s3_client.clone());

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

            let id = sqs_message.id();
            purgatory.add_task(id).await;

            match sqs_message {
                SqsMessage::Compile { request } => {
                    let compilation_input =
                        match prepare_compile_input(&request, &db_client, &s3_client).await {
                            Ok(value) => value,
                            Err(PreparationError::NoDBItemError(err)) => {
                                // Delete the message in this case. something weird.
                                // No need to cleanup s3
                                error!("{}", PreparationError::NoDBItemError(err));
                                if let Err(err) = sqs_receiver.delete_message(receipt_handle).await
                                {
                                    warn!("{}", err);
                                }
                                continue;
                            }
                            Err(PreparationError::UnexpectedStatusError(err)) => {
                                // Probably some other instance executing this at the same time.
                                // For sake of safety still try delete it. Doesn't matter if succeed
                                // No need to cleanup s3
                                info!("{}", PreparationError::UnexpectedStatusError(err));
                                if let Err(err) = sqs_receiver.delete_message(receipt_handle).await
                                {
                                    warn!("{}", err);
                                }

                                continue;
                            }
                            Err(PreparationError::VersionNotSupported(err)) => {
                                let dir = format!("{}/", id);
                                s3_client.delete_dir(&dir).await.unwrap(); // TODO: do those retriable

                                let task_result = on_compilation_failed(
                                    id,
                                    &db_client,
                                    PreparationError::VersionNotSupported(err).to_string(),
                                )
                                .await
                                .unwrap();
                                purgatory.update_task(id, task_result).await; // TODO: actually don't need anything

                                if let Err(err) = sqs_receiver.delete_message(receipt_handle).await
                                {
                                    warn!("{}", err);
                                }
                                continue;
                            }
                            Err(PreparationError::S3Error(err)) => {
                                warn!("S3Error during preparation - ignoring. {}", err);
                                continue;
                            }
                            Err(PreparationError::DBError(err)) => {
                                warn!("DBError during preparation - ignoring. {}", err);
                                continue;
                            }
                        };

                    let task_result = match do_compile(compilation_input).await {
                        Ok(value) => on_compilation_success(id, &db_client, &s3_client, value)
                            .await
                            .unwrap(), // TODO: unwraps
                        Err(err) => on_compilation_failed(id, &db_client, err.to_string())
                            .await
                            .unwrap(),
                    };
                    purgatory.update_task(id, task_result).await;

                    let dir = format!("{}/", id);
                    s3_client.delete_dir(&dir).await.unwrap();
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
