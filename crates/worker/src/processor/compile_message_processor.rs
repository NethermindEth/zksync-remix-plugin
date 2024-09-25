use std::future::Future;
use anyhow::anyhow;
use types::item::TaskResult;
use uuid::Uuid;
use crate::clients::errors::{S3Error, SqsDeleteError};

use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::commands::compile::CompilationInput;
use crate::commands::errors::{CommandResultHandleError, PreparationError};
use crate::errors::MessageProcessorError;
use crate::processor::MessageProcessor;
use crate::purgatory::Purgatory;
use crate::utils::lib::s3_compilation_files_dir;

#[derive(thiserror::Error, Debug)]
pub enum CompileProcessorError {
    #[error("PreparationError: {0}")]
    PreparationError(#[from] PreparationError),
    #[error("CommandResultHandleError: {0}")]
    CommandResultHandleError(#[from] CommandResultHandleError),
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error)
}

pub trait CompileMessageProcessor: Send + Sync + 'static {
    type Message;
    async fn process_message(
        &self,
        message: Self::Message,
    ) -> Result<TaskResult, CompileProcessorError>;
}

pub struct CompileProcessor<P, A, M, O>
where
    P: Preparator<Message = M, Output = O>,
    A: Actor<Input = P::Output>,
{
    input_preparator: P, // shall be data manager and passed in compiler(actor)
    actor: A,
    purgatory: Purgatory,
    sqs_client: SqsClientWrapper,
    s3_client: S3ClientWrapper,
}

impl<P, A, M, O> CompileProcessor<P, A, M, O> {
    async fn handle_prepare_compile_result(
        &self,
        id: Uuid,
        result: Result<O, PreparationError>,
        receipt_handle: &str,
    ) -> Result<CompilationInput, MessageProcessorError> {
        let result = match result {
            Ok(value) => Ok(value),
            Err(PreparationError::NoDBItemError(err)) => {
                // Possible in case GlobalState purges old message
                // that somehow stuck in queue for too long
                self.sqs_client.delete_message(receipt_handle).await?;
                Err(PreparationError::NoDBItemError(err))
            }
            Err(PreparationError::UnexpectedStatusError(err)) => {
                // Probably some other instance executing this at the same time.
                // For sake of safety still try to delete it. Doesn't matter if succeeds.
                // No need to clean up s3
                self.sqs_client.delete_message(receipt_handle).await.map_err(|e| anyhow!(e).into())?;
                Err(PreparationError::UnexpectedStatusError(err))
            }
            Err(PreparationError::VersionNotSupported(err)) => {
                // Clean everything since the request failed
                let dir = s3_compilation_files_dir(id.to_string().as_str());
                self.s3_client.delete_dir(&dir).await?;

                // This error doesn't create any artifacts
                let _ = self
                    .on_compilation_failed(
                        id,
                        PreparationError::VersionNotSupported(err.clone()).to_string(),
                    )
                    .await?;

                self.sqs_client.delete_message(receipt_handle).await?;
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

    async fn on_process_success(&self, id: Uuid, compilation_output: O) -> Result<TaskResult, >
}

impl<P, A, M, O> CompileMessageProcessor for CompileProcessor<P, A, M, O>
where
    P: Preparator<Message = M, Output = O>,
    A: Actor<Input = P::Output>,
{
    type Message = M;
    async fn process_message(&self, message: Self::Message) -> impl Future<Output = ()> + Send {
        let prepared_input = self.input_preparator.prepare();
    }
}

pub trait Preparator {
    type Message;
    type Output;
    fn prepare(&self, request: &Self::Request) -> Result<Self::Output, PreparationError>; // some error
}

pub trait Actor {
    type Input;
    fn act(&self, input: Self::Input) -> Result<(), ()>; // some result
}

// MainProcessor:
//  calls UserProcessor
//  gets back TaskResult maybe in Result<TaskResult, UserProcessorError>
//  handles error
//  writes result to db
//  handles call to purgatory(?) - shall be handled by
