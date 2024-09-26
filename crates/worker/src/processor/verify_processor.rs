use tracing::error;
use types::item::task_result::{TaskResult, TaskSuccess};
use types::VerificationRequest;

use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::commands::verify::do_verify;
use crate::processor::errors::VerifyProcessorError;
use crate::processor::input_preparator::VerifyInputPreparator;
use crate::purgatory::Purgatory;
use crate::utils::lib::{s3_compilation_files_dir, ZKSOLC_VERSIONS};

// TODO: make generic via adding MessageProcessor trait with process_message(...)
pub struct VerifyProcessor {
    sqs_client: SqsClientWrapper,
    s3_client: S3ClientWrapper,
    input_preparator: VerifyInputPreparator,
    purgatory: Purgatory,
}

impl VerifyProcessor {
    pub fn new(
        sqs_client: SqsClientWrapper,
        s3_client: S3ClientWrapper,
        purgatory: Purgatory,
    ) -> Self {
        let input_preparator = VerifyInputPreparator::new(s3_client.clone());
        Self {
            sqs_client,
            s3_client,
            input_preparator,
            purgatory,
        }
    }

    async fn validate_message(
        &self,
        message: &VerificationRequest,
    ) -> Result<(), VerifyProcessorError> {
        let zksolc_version = message.config.zksolc_version.as_str();
        if !ZKSOLC_VERSIONS.contains(&zksolc_version) {
            Err(VerifyProcessorError::VersionNotSupportedError(
                zksolc_version.to_string(),
            ))
        } else {
            Ok(())
        }
    }

    pub async fn process_message(
        &self,
        message: VerificationRequest,
        receipt_handle: String,
    ) -> Result<String, VerifyProcessorError> {
        let id = message.id;

        // TODO: validator accepting SqsMessage
        self.validate_message(&message).await.map_err(|err| {
            // Reckoned as independent piece
            let receipt_handle_copy = receipt_handle.clone();
            let dir = s3_compilation_files_dir(id.to_string().as_str());
            let s3_client = self.s3_client.clone();
            let sqs_client = self.sqs_client.clone();
            tokio::spawn(async move {
                if let Err(err) = s3_client.delete_dir(&dir).await {
                    error!("Couldn't delete compilation files on failed validation: {err}")
                }
                if let Err(err) = sqs_client.delete_message(receipt_handle_copy).await {
                    error!("Failed to delete message from sqs: {err}");
                }
            });

            err
        })?;

        let input = self
            .input_preparator
            .prepare_compile_input(&message)
            .await
            .map_err(|err| {
                let receipt_handle_copy = receipt_handle.clone();
                let sqs_client = self.sqs_client.clone();
                tokio::spawn(async move {
                    if let Err(err) = sqs_client.delete_message(receipt_handle_copy).await {
                        error!("Failed to delete message from sqs: {err}");
                    }
                });

                err
            })?;

        // Reckoned as independent piece
        {
            let dir = s3_compilation_files_dir(id.to_string().as_str());
            let s3_client = self.s3_client.clone();
            let sqs_client = self.sqs_client.clone();
            tokio::spawn(async move {
                // Clean compilation input files right away
                if let Err(err) = s3_client.delete_dir(&dir).await {
                    error!("Filed to delete compilation file: {err}")
                }
                if let Err(err) = sqs_client.delete_message(receipt_handle).await {
                    error!("Failed to delete message from sqs: {err}");
                }
            });
        }

        match do_verify(input).await {
            Ok(message) => {
                self.purgatory
                    .add_record(
                        id,
                        TaskResult::Success(TaskSuccess::Verify {
                            message: message.clone(),
                        }),
                    )
                    .await;
                Ok(message)
            }
            Err(err) => {
                let task_result = TaskResult::Failure((&err).into());
                self.purgatory.add_record(id, task_result).await;

                Err(err.into())
            }
        }
    }
}
