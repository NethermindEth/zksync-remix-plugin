use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{error, info, warn};
use types::SqsMessage;
use uuid::Uuid;

use crate::commands::compile::compile;
use crate::dynamodb_client::DynamoDBClient;
use crate::errors::CompilationError;
use crate::s3_client::S3Client;
use crate::sqs_client::wrapper::SqsClientWrapper;
use crate::sqs_listener::{SqsListener, SqsReceiver};
use crate::utils::lib::{timestamp, DURATION_TO_PURGE};

pub type Timestamp = u64;

pub struct EngineBuilder {
    sqs_client: SqsClientWrapper,
    db_client: DynamoDBClient,
    s3_client: S3Client,
    is_supervisor_enabled: bool,
    running_workers: Vec<RunningEngine>,
}

impl EngineBuilder {
    pub fn new(
        sqs_client: SqsClientWrapper,
        db_client: DynamoDBClient,
        s3_client: S3Client,
        supervisor_enabled: bool,
    ) -> Self {
        EngineBuilder {
            sqs_client,
            db_client,
            s3_client,
            running_workers: vec![],
            is_supervisor_enabled: supervisor_enabled,
        }
    }

    pub fn start(self, num_workers: NonZeroUsize) -> RunningEngine {
        let sqs_listener = SqsListener::new(self.sqs_client, Duration::from_millis(500));
        let s3_client = self.s3_client.clone();
        let db_client = self.db_client.clone();

        RunningEngine::new(
            sqs_listener,
            db_client,
            s3_client,
            num_workers.get(),
            self.is_supervisor_enabled,
        )
    }
}

pub struct RunningEngine {
    sqs_listener: SqsListener,
    expiration_timestamps: Arc<Mutex<Vec<(Uuid, Timestamp)>>>,
    num_workers: usize,
    worker_threads: Vec<JoinHandle<()>>,
    supervisor_thread: Option<JoinHandle<()>>,
}

impl RunningEngine {
    pub fn new(
        sqs_listener: SqsListener,
        db_client: DynamoDBClient,
        s3_client: S3Client,
        num_workers: usize,
        enable_supervisor: bool,
    ) -> Self {
        let  expiration_timestamps=  Arc::new(Mutex::new(vec![]));
        let mut worker_threads = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            // Start worker
            let sqs_receiver = sqs_listener.receiver();
            let db_client_copy = db_client.clone();
            let s3_client_copy = s3_client.clone();
            let expiration_timestamps = expiration_timestamps.clone();

            worker_threads.push(tokio::spawn(async move {
                RunningEngine::worker(
                    sqs_receiver,
                    db_client_copy,
                    s3_client_copy,
                    expiration_timestamps,
                )
                .await;
            }));
        }

        let supervisor_thread = if enable_supervisor {
            let db_client = db_client.clone();
            let expiration_timestamps = expiration_timestamps.clone();
            Some(tokio::spawn(async move {
                RunningEngine::supervisor(db_client, expiration_timestamps).await;
            }))
        } else {
            None
        };

        Self {
            sqs_listener,
            expiration_timestamps,
            num_workers,
            supervisor_thread,
            worker_threads,
        }
    }

    async fn worker(
        sqs_receiver: SqsReceiver,
        db_client: DynamoDBClient,
        s3_client: S3Client,
        expiration_timestamps: Arc<Mutex<Vec<(Uuid, Timestamp)>>>,
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
                let _ = sqs_receiver.delete_message(receipt_handle).await;
                continue;
            };

            let sqs_message = match serde_json::from_str::<SqsMessage>(&body) {
                Ok(sqs_message) => sqs_message,
                Err(err) => {
                    error!("Could not deserialize message: {}", err.to_string());
                    let _ = sqs_receiver.delete_message(receipt_handle).await;
                    continue;
                }
            };

            match sqs_message {
                SqsMessage::Compile { request } => {
                    let result = compile(request, &db_client, &s3_client).await; // TODO:
                    match result {
                        Ok(()) => {}
                        Err(err) => match err {
                            CompilationError::DBError(err) => {
                                warn!("compilation DBError: {}", err.to_string());
                                continue;
                            }
                            CompilationError::S3Error(err) => {
                                warn!("compilation S3Error: {}", err.to_string());
                                continue;
                            }
                            CompilationError::NoDBItemError(err) => {
                                warn!("{}", err.to_string());
                            }
                            CompilationError::UnexpectedStatusError(err) => {
                                warn!("{}", err.to_string());
                            }
                            CompilationError::IoError(err) => {
                                warn!("IOError: {}", err.to_string());
                            }
                            _ => error!("Unexpected branch."),
                        },
                    }
                }
                SqsMessage::Verify { request } => {} // TODO;
            }

            let _ = sqs_receiver.delete_message(receipt_handle).await;
        }
    }

    pub async fn supervisor(
        db_client: DynamoDBClient,
        expiration_timestamps: Arc<Mutex<Vec<(Uuid, Timestamp)>>>,
    ) {
        loop {
            let now = timestamp();

            let to_delete = {
                let mut to_delete = vec![];
                let mut expiration_timestamps = expiration_timestamps.lock().await;
                expiration_timestamps.retain(|&(uuid, expiration)| {
                    if expiration < now {
                        to_delete.push(uuid);
                        false
                    } else {
                        true
                    }
                });

                to_delete
            };

            for uuid in to_delete {
                db_client.delete_item(uuid.to_string()).await;
            }

            sleep(Duration::from_millis(2000)).await;
        }
    }

    pub async fn wait(self) {
        futures::future::join_all(self.worker_threads).await;
    }
}
