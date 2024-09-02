use crate::commands::compile::compile;
use crate::dynamodb_client::DynamoDBClient;
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

use crate::sqs_client::SqsClient;
use crate::sqs_listener::{SqsListener, SqsReceiver};
use crate::utils::lib::{timestamp, DURATION_TO_PURGE};

pub type Timestamp = u64;
pub struct RunningWorker {
    sqs_listener: SqsListener,
    expiration_timestamps: Arc<Mutex<Vec<(Uuid, Timestamp)>>>,
    num_workers: usize,
    worker_threads: Vec<JoinHandle<()>>,
}

impl RunningWorker {
    pub fn new(
        sqs_listener: SqsListener,
        db_client: DynamoDBClient,
        s3_client: aws_sdk_s3::Client,
        num_workers: usize,
        expiration_timestamps: Arc<Mutex<Vec<(Uuid, Timestamp)>>>,
    ) -> Self {
        let mut worker_threads = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            // Start worker
            let sqs_receiver = sqs_listener.receiver();
            let db_client_copy = db_client.clone();
            let s3_client_copy = s3_client.clone();
            let expiration_timestamps = expiration_timestamps.clone();

            worker_threads.push(tokio::spawn(async move {
                RunningWorker::worker(
                    sqs_receiver,
                    db_client_copy,
                    s3_client_copy,
                    expiration_timestamps,
                )
                .await;
            }));
        }

        Self {
            sqs_listener,
            expiration_timestamps,
            num_workers,
            worker_threads,
        }
    }

    async fn worker(
        sqs_receiver: SqsReceiver,
        db_client: DynamoDBClient,
        s3_client: aws_sdk_s3::Client,
        expiration_timestamps: Arc<Mutex<Vec<(Uuid, Timestamp)>>>,
    ) {
        // TODO: process error
        while let Ok(message) = sqs_receiver.recv().await {
            let body = if let Some(body) = message.body {
                body
            } else {
                continue;
            };

            let receipt_handle = if let Some(receipt_handle) = message.receipt_handle {
                receipt_handle
            } else {
                warn!("Has body but not handle");
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
                    let _ = compile(request, &db_client, &s3_client).await; // TODO:
                }
                SqsMessage::Verify { request } => {} // TODO;
            }

            let _ = sqs_receiver.delete_message(receipt_handle).await;
        }
    }
}

pub struct WorkerEngine {
    sqs_client: SqsClient,
    db_client: DynamoDBClient,
    s3_client: aws_sdk_s3::Client,
    expiration_timestamps: Arc<Mutex<Vec<(Uuid, Timestamp)>>>,
    is_supervisor_enabled: Arc<AtomicBool>,
    running_workers: Vec<RunningWorker>,
    supervisor_thread: Arc<Option<JoinHandle<()>>>,
}

impl WorkerEngine {
    pub fn new(
        sqs_client: SqsClient,
        db_client: DynamoDBClient,
        s3_client: aws_sdk_s3::Client,
        supervisor_enabled: bool,
    ) -> Self {
        let is_supervisor_enabled = Arc::new(AtomicBool::new(supervisor_enabled));
        let expiration_timestamps = Arc::new(Mutex::new(vec![]));

        WorkerEngine {
            sqs_client,
            db_client,
            s3_client,
            supervisor_thread: Arc::new(None),
            expiration_timestamps,
            running_workers: vec![],
            is_supervisor_enabled,
        }
    }

    pub fn start(&mut self, num_workers: NonZeroUsize) {
        let sqs_listener = SqsListener::new(self.sqs_client.clone(), Duration::from_millis(500));
        let s3_client = self.s3_client.clone();
        let db_client = self.db_client.clone();
        self.running_workers.push(RunningWorker::new(
            sqs_listener,
            db_client,
            s3_client,
            num_workers.get(),
            self.expiration_timestamps.clone(),
        ));

        // TODO: not protection really
        if self.is_supervisor_enabled.load(Ordering::Acquire) && self.supervisor_thread.is_none() {
            let db_client = self.db_client.clone();
            let expiration_timestamps = self.expiration_timestamps.clone();
            self.supervisor_thread = Arc::new(Some(tokio::spawn(async move {
                WorkerEngine::supervisor(db_client, expiration_timestamps).await;
            })));
        }
    }

    // pub async fn enable_supervisor_thread(&mut self) {
    //     if self.supervisor_thread.is_some() {
    //         return;
    //     }
    //
    //     self.is_supervisor_enabled.store(true, Ordering::Release);
    //     let expiration_timestamps = self.expiration_timestamps.clone();
    //
    //     self.supervisor_thread = Arc::new(Some(tokio::spawn(async move {
    //         WorkerEngine::supervisor(self.db_client.clone(), expiration_timestamps).await;
    //     })));
    // }

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

    // pub async fn disable_supervisor_thread(&mut self) {
    //     let mut is_enabled = self.is_supervisor_enabled.lock().await;
    //     *is_enabled = false;
    //
    //     if let Ok(Some(join_handle)) = Arc::try_unwrap(self.supervisor_thread.clone()) {
    //         let _ = join_handle.await;
    //     }
    //
    //     self.supervisor_thread = Arc::new(None);
    // }
}
