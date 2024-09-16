use std::num::NonZeroUsize;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use types::{CompilationRequest, SqsMessage, VerificationRequest};

use crate::clients::dynamodb_client::DynamoDBClient;
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::commands::compile::{do_compile, CompilationInput};
use crate::commands::errors::PreparationError;
use crate::commands::utils::{
    on_compilation_failed, on_compilation_success, prepare_compile_input,
};
use crate::errors::MessageProcessorError;
use crate::purgatory::Purgatory;
use crate::sqs_listener::{SqsListener, SqsReceiver};
use crate::utils::lib::s3_compilation_files_dir;

pub struct EngineBuilder {
    sqs_client: SqsClientWrapper,
    db_client: DynamoDBClient,
    s3_client: S3ClientWrapper,
    running_workers: Vec<RunningEngine>,
}

impl EngineBuilder {
    pub fn new(
        sqs_client: SqsClientWrapper,
        db_client: DynamoDBClient,
        s3_client: S3ClientWrapper,
    ) -> Self {
        EngineBuilder {
            sqs_client,
            db_client,
            s3_client,
            running_workers: vec![],
        }
    }

    pub fn start(self, num_workers: NonZeroUsize) -> RunningEngine {
        let sqs_listener = SqsListener::new(self.sqs_client, Duration::from_millis(500));

        RunningEngine::new(
            sqs_listener,
            self.db_client,
            self.s3_client,
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
        s3_client: S3ClientWrapper,
        num_workers: usize,
    ) -> Self {
        let purgatory = Purgatory::new(db_client.clone(), s3_client.clone());

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
        s3_client: S3ClientWrapper,
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

            // TODO: add metrics for how long it takes -
            // adjust "visibility timeout" or receiver chan capacity
            match sqs_message {
                SqsMessage::Compile { request } => {
                    let result = Self::process_compile_message(
                        request,
                        receipt_handle,
                        &sqs_receiver,
                        &db_client,
                        &s3_client,
                        &mut purgatory,
                    )
                    .await;
                    if let Err(err) = result {
                        error!("{}", err);
                    }
                }
                SqsMessage::Verify { request } => {
                    Self::process_verify_message(request, receipt_handle, &sqs_receiver).await
                }
            }
        }
    }

    // TODO(future me): could return bool.
    async fn process_compile_message(
        request: CompilationRequest,
        receipt_handle: String,
        sqs_receiver: &SqsReceiver,
        db_client: &DynamoDBClient,
        s3_client: &S3ClientWrapper,
        purgatory: &mut Purgatory,
    ) -> Result<(), MessageProcessorError> {
        let compilation_input = Self::handle_prepare_compile_input(
            &request,
            &receipt_handle,
            sqs_receiver,
            db_client,
            s3_client,
        )
        .await?;

        let id = request.id;
        let task_result = match do_compile(compilation_input).await {
            Ok(value) => on_compilation_success(id, &db_client, &s3_client, value).await?,
            Err(err) => on_compilation_failed(id, &db_client, err.to_string()).await?,
        };
        purgatory.add_record(id, task_result).await;

        // Clean compilation input files right away
        let dir = s3_compilation_files_dir(id.to_string().as_str());
        s3_client.delete_dir(&dir).await?;

        sqs_receiver.delete_message(receipt_handle).await?;
        Ok(())
    }

    // TODO(future me): extract in a class
    pub(crate) async fn handle_prepare_compile_input(
        request: &CompilationRequest,
        receipt_handle: &str,
        sqs_receiver: &SqsReceiver,
        db_client: &DynamoDBClient,
        s3_client: &S3ClientWrapper,
    ) -> Result<CompilationInput, MessageProcessorError> {
        let id = request.id;
        let result = match prepare_compile_input(&request, db_client, s3_client).await {
            Ok(value) => Ok(value),
            Err(PreparationError::NoDBItemError(err)) => {
                // Possible in case GlobalState purges old message
                // that somehow stuck in queue for too long
                sqs_receiver.delete_message(receipt_handle).await?;
                Err(PreparationError::NoDBItemError(err))
            }
            Err(PreparationError::UnexpectedStatusError(err)) => {
                // Probably some other instance executing this at the same time.
                // For sake of safety still try to delete it. Doesn't matter if succeeds.
                // No need to clean up s3
                sqs_receiver.delete_message(receipt_handle).await?;
                Err(PreparationError::UnexpectedStatusError(err))
            }
            Err(PreparationError::VersionNotSupported(err)) => {
                // Clean everything since the request failed
                let dir = s3_compilation_files_dir(id.to_string().as_str());
                s3_client.delete_dir(&dir).await?;

                // This error doesn't create any artifacts
                let _ = on_compilation_failed(
                    id,
                    &db_client,
                    PreparationError::VersionNotSupported(err.clone()).to_string(),
                )
                .await?;

                sqs_receiver.delete_message(receipt_handle).await?;
                Err(PreparationError::VersionNotSupported(err))
            }
            Err(PreparationError::S3Error(err)) => {
                // Certain cases don't require delete_message
                Err(PreparationError::S3Error(err))
            }
            Err(PreparationError::DBError(err)) => {
                // Certain cases don't require delete_message
                Err(PreparationError::DBError(err))
            }
        };

        result.map_err(MessageProcessorError::from)
    }

    async fn process_verify_message(
        request: VerificationRequest,
        receipt_handle: String,
        sqs_receiver: &SqsReceiver,
    ) {
        // TODO: implement

        if let Err(err) = sqs_receiver.delete_message(receipt_handle).await {
            warn!("{}", err);
        }
    }

    pub async fn wait(self) {
        futures::future::join_all(self.worker_threads).await;
    }
}
