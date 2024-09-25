use anyhow::{anyhow, Context};
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use tracing::{error};
use types::item::{Item, Status, TaskResult};
use types::{SqsMessage, VerificationRequest};
use uuid::Uuid;

use crate::clients::dynamodb_clients::wrapper::DynamoDBClientWrapper;
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::compile_processor::CompileProcessor;
use crate::purgatory::Purgatory;

// TODO: generic in the future, handling specific message type- chain dependant.
pub struct Processor {
    db_client: DynamoDBClientWrapper,
    s3_client: S3ClientWrapper,
    sqs_client: SqsClientWrapper,
    compile_processor: CompileProcessor,
    purgatory: Purgatory,
}

impl Processor {
    pub fn new(
        db_client: DynamoDBClientWrapper,
        s3_client: S3ClientWrapper,
        sqs_client: SqsClientWrapper,
        compile_processor: CompileProcessor,
        purgatory: Purgatory,
    ) -> Self {
        Self {
            db_client,
            compile_processor,
            sqs_client,
            s3_client,
            purgatory,
        }
    }

    async fn lock_item(&self, id: Uuid) -> anyhow::Result<()> {
        self.db_client
            .update_item_status_conditional(
                id.to_string().as_str(),
                &Status::Pending,
                &Status::InProgress,
            )
            .await
            .map_err(|err| match err {
                SdkError::ServiceError(err) => match err.into_err() {
                    UpdateItemError::ConditionalCheckFailedException(err) => {
                        anyhow!("Couldn't lock the item: {}", err)
                    }
                    err => anyhow!(err),
                },
                err => anyhow!(err),
            })
    }

    // TODO(future me): could return bool.
    pub async fn process_message(
        &self,
        sqs_message: SqsMessage,
        receipt_handle: String,
    ) -> anyhow::Result<()> {
        let id = sqs_message.id();
        self.lock_item(id).await.or_else(|err| {
            let sqs_client = self.sqs_client.clone();
            let receipt_handle_copy = receipt_handle.clone();
            tokio::spawn(async move {
                // That could be due to wrong Status or no item
                // 1. No item is possible in case GlobalState purges old message - delete from sqs
                // 2. Wrong Status - other instance picked this up.
                // For sake of safety still try to delete it. Doesn't matter if succeeds.
                if let Err(err) = sqs_client.delete_message(receipt_handle_copy).await {
                    error!("Couldn't delete sqs message: {err}")
                }
            });

            Err(err)
        })?;

        let task_result = match sqs_message {
            SqsMessage::Compile { request } => {
                let result = self
                    .compile_processor
                    .process_message(request, receipt_handle)
                    .await;

                match result {
                    Ok(val) => TaskResult::Success {
                        presigned_urls: val,
                    },
                    Err(err) => TaskResult::Failure(err.to_string()),
                }
            }
            SqsMessage::Verify { request } => {
                self.process_verify_request(request, receipt_handle).await
            }
        };

        self.handle_task_result(id, task_result).await
    }

    async fn process_verify_request(
        &self,
        request: VerificationRequest,
        receipt_handle: String,
    ) -> TaskResult {
        // TODO: implement

        // if let Err(err) = self.sqs_receiver.delete_message(receipt_handle).await {
        //     warn!("{}", err);
        // }

        todo!()
    }

    async fn handle_task_result(&self, id: Uuid, task_result: TaskResult) -> anyhow::Result<()> {
        let mut builder = self
            .db_client
            .client
            .client
            .update_item()
            .table_name(self.db_client.client.table_name.clone())
            .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
            .update_expression("SET #status = :newStatus, #data = :data")
            .expression_attribute_names("#status", Status::attribute_name())
            .expression_attribute_names("#data", Item::data_attribute_name());

        builder = match task_result {
            TaskResult::Success { presigned_urls } => {
                builder
                    .expression_attribute_values(
                        ":newStatus",
                        AttributeValue::N(2.to_string()), // Ready
                    )
                    .expression_attribute_values(":data", AttributeValue::Ss(presigned_urls))
            }
            TaskResult::Failure(message) => {
                builder
                    .expression_attribute_values(
                        ":newStatus",
                        AttributeValue::N(3.to_string()), // Failed
                    )
                    .expression_attribute_values(":data", AttributeValue::S(message.clone()))
            }
        };

        self.db_client
            .update_item_raw(&builder)
            .await
            .map_err(anyhow::Error::from)
            .with_context(|| "Couldn't write task result")
    }
}
