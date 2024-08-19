use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{error::SdkError, operation::put_item::PutItemError};
use lambda_http::{
    run, service_fn, Error as LambdaError, Request as LambdaRequest, Response as LambdaResponse,
};
use serde::Deserialize;
use std::ops::Add;
use tracing::{error, info};

mod common;
use crate::common::{errors::Error, utils::extract_request, Item, Status, BUCKET_NAME_DEFAULT};

const QUEUE_URL_DEFAULT: &str = "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";
const TABLE_NAME_DEFAULT: &str = "zksync-table";

const NO_OBJECTS_TO_COMPILE_ERROR: &str = "There are no objects to compile";

#[derive(Debug, Deserialize)]
struct Request {
    pub id: String,
}

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

async fn compile(
    id: String,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    sqs_client: &aws_sdk_sqs::Client,
    queue_url: &str,
) -> Result<(), Error> {
    let item = Item {
        id: id.clone(),
        status: Status::Pending,
    };

    let result = dynamo_client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item.into()))
        .condition_expression("attribute_not_exists(ID)")
        .send()
        .await;

    match result {
        Ok(val) => val,
        Err(SdkError::ServiceError(val)) => match val.err() {
            PutItemError::ConditionalCheckFailedException(_) => {
                error!("Recompilation attempt, id: {}", id);
                let response = lambda_http::Response::builder()
                    .status(400)
                    .header("content-type", "text/html")
                    .body("Recompilation attempt".into())
                    .map_err(Error::from)?;

                return Err(Error::HttpError(response));
            }
            _ => return Err(Box::new(SdkError::ServiceError(val)).into()),
        },
        Err(err) => return Err(Box::new(err).into()),
    };

    let message_output = sqs_client
        .send_message()
        .queue_url(queue_url)
        .message_body(id)
        .send()
        .await
        .map_err(Box::new)?;

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
) -> Result<LambdaResponse<String>, Error> {
    let request = extract_request::<Request>(request)?;

    let objects = s3_client
        .list_objects_v2()
        .delimiter('/')
        .prefix(request.id.clone().add("/"))
        .bucket(bucket_name)
        .send()
        .await
        .map_err(Box::new)?;

    if let None = &objects.contents {
        error!("No objects in folder: {}", request.id);
        let response = LambdaResponse::builder()
            .status(400)
            .header("content-type", "text/html")
            .body(NO_OBJECTS_TO_COMPILE_ERROR.into())
            .map_err(Error::from)?;

        return Err(Error::HttpError(response));
    }

    info!("Compile");
    compile(request.id, dynamo_client, table_name, sqs_client, queue_url).await?;

    let response = LambdaResponse::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(Default::default())
        .map_err(Box::new)?;

    return Ok(response);
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
            Ok(val) => Ok(val),
            Err(Error::HttpError(val)) => Ok(val),
            Err(Error::LambdaError(err)) => Err(err),
        }
    }))
    .await
}
