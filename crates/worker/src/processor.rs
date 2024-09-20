use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;
use tracing::{error, warn};
use types::item::{Item, Status, TaskResult};
use types::{CompilationRequest, SqsMessage, VerificationRequest, ARTIFACTS_FOLDER};
use uuid::Uuid;

use crate::clients::dynamodb_clients::wrapper::DynamoDBClientWrapper;
use crate::clients::errors::{DBError, S3Error};
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::commands::compile::{do_compile, CompilationInput, CompilationOutput};
use crate::commands::errors::{CommandResultHandleError, PreparationError};
use crate::errors::MessageProcessorError;
use crate::input_preparator::InputPreparator;
use crate::purgatory::Purgatory;
use crate::sqs_listener::SqsReceiver;
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::lib::s3_compilation_files_dir;

// TODO: generic in the future, handling specific message type- chain dependant.
pub struct Processor<'a> {
    sqs_receiver: &'a SqsReceiver,
    db_client: &'a DynamoDBClientWrapper,
    s3_client: &'a S3ClientWrapper,
    purgatory: &'a mut Purgatory,
}

impl<'a> Processor<'a> {
    pub fn new(
        sqs_receiver: &'a SqsReceiver,
        db_client: &'a DynamoDBClientWrapper,
        s3_client: &'a S3ClientWrapper,
        purgatory: &'a mut Purgatory,
    ) -> Self {
        Self {
            sqs_receiver,
            db_client,
            s3_client,
            purgatory,
        }
    }

    pub async fn process_message(&mut self, sqs_message: SqsMessage, receipt_handle: String) {
        match sqs_message {
            SqsMessage::Compile { request } => {
                let result = self.process_compile_request(request, &receipt_handle).await;
                if let Err(err) = result {
                    error!("{}", err);
                }
            }
            SqsMessage::Verify { request } => {
                self.process_verify_request(request, receipt_handle).await
            }
        }
    }

    async fn process_verify_request(&self, request: VerificationRequest, receipt_handle: String) {
        // TODO: implement

        if let Err(err) = self.sqs_receiver.delete_message(receipt_handle).await {
            warn!("{}", err);
        }
    }

    // TODO(future me): could return bool.
    async fn process_compile_request(
        &mut self,
        request: CompilationRequest,
        receipt_handle: &str, // TODO; &str changes
    ) -> Result<(), MessageProcessorError> {
        let input_preparator = InputPreparator::new(self.db_client, self.s3_client);
        let preparation_result = input_preparator.prepare_compile_input(&request).await;

        let id = request.id;
        let compilation_input = self
            .handle_prepare_compile_result(id, preparation_result, receipt_handle)
            .await?;
        let task_result = match do_compile(compilation_input).await {
            Ok(value) => self.on_compilation_success(id, value).await?,
            Err(err) => self.on_compilation_failed(id, err.to_string()).await?,
        };
        self.purgatory.add_record(id, task_result).await;

        // Clean compilation input files right away
        let dir = s3_compilation_files_dir(id.to_string().as_str());
        self.s3_client.delete_dir(&dir).await?;

        self.sqs_receiver.delete_message(receipt_handle).await?;

        Ok(())
    }

    async fn handle_prepare_compile_result(
        &self,
        id: Uuid,
        result: Result<CompilationInput, PreparationError>,
        receipt_handle: &str,
    ) -> Result<CompilationInput, MessageProcessorError> {
        let result = match result {
            Ok(value) => Ok(value),
            Err(PreparationError::NoDBItemError(err)) => {
                // Possible in case GlobalState purges old message
                // that somehow stuck in queue for too long
                self.sqs_receiver.delete_message(receipt_handle).await?;
                Err(PreparationError::NoDBItemError(err))
            }
            Err(PreparationError::UnexpectedStatusError(err)) => {
                // Probably some other instance executing this at the same time.
                // For sake of safety still try to delete it. Doesn't matter if succeeds.
                // No need to clean up s3
                self.sqs_receiver.delete_message(receipt_handle).await?;
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

                self.sqs_receiver.delete_message(receipt_handle).await?;
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

    async fn on_compilation_success(
        &self,
        id: Uuid,
        compilation_output: CompilationOutput,
    ) -> Result<TaskResult, CommandResultHandleError> {
        const DOWNLOAD_URL_EXPIRATION: Duration = Duration::from_secs(5 * 60 * 60);

        let auto_clean_up = AutoCleanUp {
            dirs: vec![compilation_output.artifacts_dir.to_str().unwrap()],
        };

        let mut presigned_urls = Vec::with_capacity(compilation_output.artifacts_data.len());
        for el in compilation_output.artifacts_data {
            let absolute_path = compilation_output.artifacts_dir.join(&el.file_path);
            let file_content = tokio::fs::File::open(absolute_path).await?;

            let file_key = format!(
                "{}/{}/{}",
                ARTIFACTS_FOLDER,
                id,
                el.file_path.to_str().unwrap()
            );
            self.s3_client.put_object(&file_key, file_content).await?;

            let expires_in = PresigningConfig::expires_in(DOWNLOAD_URL_EXPIRATION).unwrap();
            let presigned_request = self
                .s3_client
                .get_object_presigned(&file_key, &expires_in)
                .await
                .map_err(S3Error::from)?;

            presigned_urls.push(presigned_request.uri().to_string());
        }

        if presigned_urls.is_empty() {
            // TODO: AttributeValue::Ss doesn't allow empty arrays. Decide what to do. for now
            presigned_urls.push("".to_string());
        }

        let builder = self
            .db_client
            .client
            .client
            .update_item()
            .table_name(self.db_client.client.table_name.clone())
            .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
            .update_expression("SET #status = :newStatus, #data = :data")
            .expression_attribute_names("#status", Status::attribute_name())
            .expression_attribute_names("#data", Item::data_attribute_name())
            .expression_attribute_values(
                ":newStatus",
                AttributeValue::N(2.to_string()), // Ready
            )
            .expression_attribute_values(":data", AttributeValue::Ss(presigned_urls.clone()));

        self.db_client
            .update_item_raw(&builder)
            .await
            .map_err(DBError::from)?;

        auto_clean_up.clean_up().await;
        Ok(TaskResult::Success { presigned_urls })
    }

    async fn on_compilation_failed(
        &self,
        id: Uuid,
        message: String,
    ) -> Result<TaskResult, CommandResultHandleError> {
        let builder = self
            .db_client
            .client
            .client
            .update_item()
            .table_name(self.db_client.client.table_name.clone())
            .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
            .update_expression("SET #status = :newStatus, #data = :data")
            .expression_attribute_names("#status", Status::attribute_name())
            .expression_attribute_names("#data", Item::data_attribute_name())
            .expression_attribute_values(
                ":newStatus",
                AttributeValue::N(3.to_string()), // Failed
            )
            .expression_attribute_values(":data", AttributeValue::S(message.clone()));

        self.db_client
            .update_item_raw(&builder)
            .await
            .map_err(DBError::from)?;

        Ok(TaskResult::Failure(message))
    }
}
