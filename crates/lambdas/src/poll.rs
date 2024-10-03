use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::http::StatusCode;
use lambda_http::{
    run, service_fn, Error as LambdaError, Request as LambdaRequest, Response as LambdaResponse,
};
use serde::Deserialize;
use tracing::error;
use types::item::errors::ItemError;
use types::item::task_result::TaskResult;
use types::item::{Item, Status};
use uuid::Uuid;

const TABLE_NAME_DEFAULT: &str = "zksync-table";
const NO_SUCH_ITEM: &str = "No such item";

mod common;
use crate::common::{errors::Error, utils::extract_request};

#[derive(Deserialize)]
struct PollRequest {
    pub id: Uuid,
}

#[tracing::instrument(skip(dynamo_client, table_name))]
async fn process_request(
    request: LambdaRequest,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
) -> Result<LambdaResponse<String>, Error> {
    let request = extract_request::<PollRequest>(&request)?;
    let output = dynamo_client
        .get_item()
        .table_name(table_name)
        .key(
            Item::primary_key_name(),
            AttributeValue::S(request.id.to_string()),
        )
        .send()
        .await
        .map_err(Box::new)?;

    let raw_item = output.item.ok_or_else(|| {
        let response = LambdaResponse::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/html")
            .body(NO_SUCH_ITEM.to_string())
            .map_err(Error::from);

        match response {
            Ok(response) => Error::HttpError(response),
            Err(err) => err,
        }
    })?;

    let item: Item = raw_item.try_into().map_err(|err: ItemError| {
        error!("Failed to deserialize item. id: {}", request.id);
        let response = LambdaResponse::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("content-type", "text/html")
            .body(err.to_string())
            .map_err(Error::from);

        match response {
            Ok(response) => Error::HttpError(response),
            Err(err) => err,
        }
    })?;

    let task_result = if let Status::Done(task_result) = item.status {
        Ok(task_result)
    } else {
        let response = LambdaResponse::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "text/html")
            .body("Task isn't ready".to_owned())
            .map_err(Error::from)?;

        Err(Error::HttpError(response))
    }?;

    match task_result {
        TaskResult::Success(value) => {
            let response = LambdaResponse::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&value)?)?;

            Ok(response)
        }
        TaskResult::Failure(value) => {
            let status_code: StatusCode = value.error_type.into();
            let response = LambdaResponse::builder()
                .status(status_code)
                .header("content-type", "text/html")
                .body(value.message)
                .map_err(Box::new)?;

            Err(Error::HttpError(response))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .without_time() // CloudWatch will add the ingestion time
        .with_target(false)
        .init();

    let table_name = std::env::var("TABLE_NAME").unwrap_or(TABLE_NAME_DEFAULT.into());

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_client = aws_sdk_dynamodb::Client::new(&config);

    run(service_fn(|request: LambdaRequest| async {
        let result = process_request(request, &dynamo_client, &table_name).await;

        match result {
            Ok(val) => Ok(val),
            Err(Error::HttpError(val)) => Ok(val),
            Err(Error::LambdaError(err)) => Err(err),
        }
    }))
    .await
}
