use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{error::SdkError, operation::put_item::PutItemError};
use chrono::Utc;
use lambda_http::http::StatusCode;
use lambda_http::{
    run, service_fn, Error as LambdaError, Request as LambdaRequest, Response as LambdaResponse,
};
use std::ops::Add;
use tracing::{error, info};
use types::{
    item::{Item, Status},
    CompilationRequest, SqsMessage,
};

mod common;
use crate::common::utils::{error_string_to_json, ExtractRequestError};
use crate::common::{utils::extract_request, BUCKET_NAME_DEFAULT};

// TODO: remove on release. random change
const QUEUE_URL_DEFAULT: &str = "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";
const TABLE_NAME_DEFAULT: &str = "zksync-table";

const NO_OBJECTS_TO_COMPILE_ERROR: &str = "There are no objects to compile";
const RECOMPILATION_ATTEMPT_ERROR: &str = "Recompilation attempt";
// impl Deserialize for Response {
//     fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
//         todo!()
//     }
// }
// TODO:
// struct SqsClient {
//     pub client: aws_sdk_sqs::Client,
//     pub queue_url: String,
//    // pub other_data: String
// }

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("{0}")]
    BadRequestError(String),
    #[error("{0}")]
    ConflictError(String),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

impl From<ExtractRequestError> for Error {
    fn from(value: ExtractRequestError) -> Self {
        Self::BadRequestError(value.to_string())
    }
}

impl TryInto<LambdaResponse<String>> for Error {
    type Error = LambdaError;
    fn try_into(self) -> Result<LambdaResponse<String>, Self::Error> {
        match self {
            Self::BadRequestError(err) => {
                let response = LambdaResponse::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(error_string_to_json(&err).to_string())?;
                Ok(response)
            }
            Self::ConflictError(err) => {
                let response = LambdaResponse::builder()
                    .status(StatusCode::CONFLICT)
                    .header("Content-Type", "application/json")
                    .body(error_string_to_json(&err).to_string())?;
                Ok(response)
            }
            Self::InternalError(err) => {
                let response = LambdaResponse::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(error_string_to_json(&err).to_string())?;
                Ok(response)
            }
        }
    }
}

async fn compile(
    request: CompilationRequest,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    sqs_client: &aws_sdk_sqs::Client,
    queue_url: &str,
) -> Result<(), Error> {
    let created_at = Utc::now();
    let item = Item {
        id: request.id,
        status: Status::Pending,
        created_at,
    };

    let result = dynamo_client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item.into()))
        .condition_expression("attribute_not_exists(ID)")
        .send()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(SdkError::ServiceError(val)) => match val.into_err() {
            PutItemError::ConditionalCheckFailedException(_) => {
                error!("Recompilation attempt, id: {}", request.id);
                Err(Error::ConflictError(
                    RECOMPILATION_ATTEMPT_ERROR.to_string(),
                ))
            }
            err => Err(Error::InternalError(err.into())),
        },
        Err(err) => Err(Error::InternalError(err.into())),
    }?;

    let message = SqsMessage::Compile { request };
    let message = match serde_json::to_string(&message) {
        Ok(val) => Ok(val),
        Err(err) => {
            error!("Serialization failed, id: {:?}", message);
            Err(anyhow::Error::from(err))
        }
    }?;

    let message_output = sqs_client
        .send_message()
        .queue_url(queue_url)
        .message_body(message)
        .send()
        .await
        .map_err(anyhow::Error::from)?;

    info!(
        "message sent to sqs. message_id: {}",
        message_output.message_id.unwrap_or("empty_id".into())
    );

    Ok(())
}

#[tracing::instrument(skip(
    dynamo_client,
    table_name,
    sqs_client,
    queue_url,
    s3_client,
    bucket_name
))]
async fn process_request(
    request: LambdaRequest,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    sqs_client: &aws_sdk_sqs::Client,
    queue_url: &str,
    s3_client: &aws_sdk_s3::Client,
    bucket_name: &str,
) -> Result<(), Error> {
    let request = extract_request::<CompilationRequest>(&request)?;
    let objects = s3_client
        .list_objects_v2()
        .bucket(bucket_name)
        .prefix(request.id.to_string().add("/"))
        .send()
        .await
        .map_err(anyhow::Error::from)?;
    objects.contents.ok_or_else(|| {
        error!("No objects in folder: {}", request.id);
        Error::BadRequestError(NO_OBJECTS_TO_COMPILE_ERROR.to_string())
    })?;

    compile(request, dynamo_client, table_name, sqs_client, queue_url).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .without_time() // CloudWatch will add the ingestion time
        .with_target(false)
        .init();

    let queue_url = std::env::var("QUEUE_URL").unwrap_or(QUEUE_URL_DEFAULT.into());
    let table_name = std::env::var("TABLE_NAME").unwrap_or(TABLE_NAME_DEFAULT.into());
    let bucket_name = std::env::var("BUCKET_NAME").unwrap_or(BUCKET_NAME_DEFAULT.into());

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_client = aws_sdk_dynamodb::Client::new(&config);
    let sqs_client = aws_sdk_sqs::Client::new(&config);
    let s3_client = aws_sdk_s3::Client::new(&config);

    run(service_fn(|request: LambdaRequest| async {
        let result = process_request(
            request,
            &dynamo_client,
            &table_name,
            &sqs_client,
            &queue_url,
            &s3_client,
            &bucket_name,
        )
        .await;

        match result {
            Ok(_) => {
                let response = LambdaResponse::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Default::default())
                    .map_err(Box::new)?;

                Ok(response)
            }
            Err(err) => Ok::<_, LambdaError>(err.try_into()?),
        }
    }))
    .await
}
