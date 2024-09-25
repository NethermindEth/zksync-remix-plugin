mod compile_message_processor;
mod user_defined_processor;

use crate::clients::dynamodb_clients::client::DynamoDBClient;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_sqs::types::Message;
use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;
use tracing::{error, warn};
use types::item::{Item, Status, TaskResult};
use types::{CompilationRequest, SqsMessage, VerificationRequest, ARTIFACTS_FOLDER};
use uuid::Uuid;

use crate::clients::dynamodb_clients::wrapper::DynamoDBClientWrapper;
use crate::clients::errors::{DBError, S3Error};
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::commands::compile::{do_compile, CompilationInput, CompilationOutput};
use crate::commands::errors::{CommandResultHandleError, PreparationError};
use crate::errors::MessageProcessorError;
use crate::input_preparator::InputPreparator;
use crate::purgatory::Purgatory;
use crate::sqs_listener::SqsReceiver;
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::lib::s3_compilation_files_dir;

// TODO: generic in the future, handling specific message type- chain dependant.

#[derive(Clone)]
pub struct Processor {
    sqs_client: SqsClientWrapper,
    db_client: DynamoDBClientWrapper,
    s3_client: S3ClientWrapper,
    purgatory: Purgatory,
}

impl Processor {
    pub fn new(
        sqs_client: SqsClientWrapper,
        db_client: DynamoDBClientWrapper,
        s3_client: S3ClientWrapper,
        purgatory: Purgatory,
    ) -> Self {
        Self {
            sqs_client,
            db_client,
            s3_client,
            purgatory,
        }
    }

    pub async fn process_sqs_message(&self, sqs_message: SqsMessage, receipt_handle: String) {
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

        if let Err(err) = self.sqs_client.delete_message(receipt_handle).await {
            warn!("{}", err);
        }
    }

    // TODO(future me): could return bool.
    async fn process_compile_request(
        &self,
        request: CompilationRequest,
        receipt_handle: &str, // TODO; &str changes
    ) -> Result<(), MessageProcessorError> {
        // 1. prepare input
        // 2. some errors require custom handling, but if fail need
        // 3. returns if need to update data in db as response - do
        // 4. compile
        // 5. on result update db with result

        // once know item to updatem
        let input_preparator = InputPreparator::new(&self.db_client, &self.s3_client);
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

        self.sqs_client.delete_message(receipt_handle).await?;

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
                self.sqs_client.delete_message(receipt_handle).await?;
                Err(PreparationError::NoDBItemError(err))
            }
            Err(PreparationError::UnexpectedStatusError(err)) => {
                // Probably some other instance executing this at the same time.
                // For sake of safety still try to delete it. Doesn't matter if succeeds.
                // No need to clean up s3
                self.sqs_client.delete_message(receipt_handle).await?;
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

pub trait MessageProcessor: Send + Sync + 'static {
    type Message;
    fn process_message(&self, message: Message) -> impl Future<Output = ()> + Send;
}

pub struct MainProcessor {
    purgatory: Purgatory,
}

impl MessageProcessor for MainProcessor {
    type Message = SqsMessage;
    fn process_message(&self, message: SqsMessage) -> impl Future<Output = ()> + Send {
        let result = match message {
            SqsMessage::Compile { request } => {}
            SqsMessage::Verify { request } => {}
        };
    }
}

// overall process
// get message
// process message
// processor may: compile

// idea is to have as much as possible reusable parts
// So we may have different SqsMessages type: Some additional variants: i.e Deploy
// The idea is to just implement that part and reuse as much as possible
// All of them will use our AWS, so that shalln't be generic. If message failed to be processed
// We can update AWS with some ApiError

// Compile does something, Deploy does something and they shalln't know anything about AWS
// Errors must be propagated below and written into our AWS. So it will be just to implement deplou
// functionality and plug it in

// Preparator maybe different - Some require s3 whole dir read, another single file or nothing at all or db reads
//

// so the idea is to have set of defined processor for variants: Compile, Verify, Deplou
// the idea is that most of them will be reusable

// Each Individual message processor would require some set of templates: Preparator
// pub struct CompileMessageProcessor<P, C> {
//     input_preparator: P,
//     compiler: C
// }
// Note: above implements custom logic related to itself: compile needs to clean after itself

// pub struct DeployMessageProcessor<P, D> {
//     input_preparator: P,
//     deployer: D
// }
// Note: Deploy doesn't need to clean

// for a particular SqsMessage there's a defined mapping to those types

// So writing new plugin would be also could reuse here
// let processor = new MessageProcessor<SqsMessage>
// pub struct MessageDispatcher<SqsMessage> {
//     compile_processor: CompileNessageProcessor<compile::StandartPreparator, ScrollCompiler>,
//     verify_processor: VerifyMessageProcessor<verify::StandartPreparator, ScrollVerifier>
// }

// pub trait Preparator {
//     type Output;
//     fn prepare(&self) -> Result<Self::Output, ()>; // some error
// }
//
// pub trait Actor {
//     type Input;
//     fn act(&self, input: Self::Input) -> Result<(), ()>; // some result
// }
//
// pub struct MessageProcessor2<P, A, T>
// where
//     P: Preparator<Output = T>,
//     A: Actor<Input = P::Output>,
// {
//     input_preparator: P,
//     actor: A,
// }
//
// impl<P, A, T> MessageProcessor2<P, A, T>
// where
//     P: Preparator<Output = T>,
//     A: Actor<Input = P::Output>,
// {
//     fn process_message(&self) {
//         let asd = self.input_preparator.prepare().unwrap();
//         self.actor.act(asd).unwrap();
//     }
// }

pub trait MessageDispatcher<M> {
    fn dispatch(&self, message: M) -> Result<i32, u32>; // Result<OkResponse, ApiError>;
}

pub struct ZkSyncDispatcher {}

pub struct MessageProcessor3<M, D: MessageDispatcher<M>> {
    db_client: DynamoDBClient,
    dispatcher: D,
}

impl<M, D: MessageDispatcher<M>> MessageProcessor for MessageProcessor3<M, D> {
    type Message = M;
    fn process_message(&self, raw_message: Message) -> impl Future<Output = ()> + Send {
        let result = self.dispatcher.dispatch(raw_message);
        self.db_client.update_with_result(result);
    }
}
