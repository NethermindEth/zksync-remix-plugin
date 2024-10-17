use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::http::StatusCode;
use lambda_http::{
    run, service_fn, Error as LambdaError, Request as LambdaRequest, Response as LambdaResponse,
    Response,
};
use serde::Deserialize;
use tracing::{error, info, warn};
use types::item::errors::ItemError;
use types::item::{Item, Status};
use uuid::Uuid;

const TABLE_NAME_DEFAULT: &str = "zksync-table";
const NO_SUCH_ITEM: &str = "No such item";

mod common;
use crate::common::errors::Error;
use crate::common::utils::error_string_to_json;

#[derive(Deserialize)]
struct PollRequest {
    pub id: Uuid,
}

#[tracing::instrument(skip(request))]
fn extract_uuid_from_request(request: &LambdaRequest) -> Result<Uuid, Error> {
    let path = request.uri().path();
    let path_segments: Vec<&str> = path.split('/').collect();
    if let Some(uuid_str) = path_segments.last() {
        match Uuid::parse_str(uuid_str) {
            Ok(uuid) => Ok(uuid),
            Err(_) => {
                info!("User supplied invalid uuid: {}", uuid_str);
                let response = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(error_string_to_json("Invalid UUID format").to_string())
                    .map_err(Box::new)?;

                return Err(Error::HttpError(response));
            }
        }
    } else {
        info!("Invalid user request: {}", path);
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(error_string_to_json("Invalid url").to_string())
            .map_err(Box::new)?;

        return Err(Error::HttpError(response));
    }
}

#[tracing::instrument(skip(dynamo_client, table_name))]
async fn process_request(
    request: LambdaRequest,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
) -> Result<LambdaResponse<String>, Error> {
    let request = PollRequest {
        id: extract_uuid_from_request(&request)?,
    };

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
        info!("Requested non existing item: {}", request.id);
        let response = LambdaResponse::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(error_string_to_json(NO_SUCH_ITEM).to_string())
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
            .header("Content-Type", "application/json")
            .body(error_string_to_json(&err).to_string())
            .map_err(Error::from);

        match response {
            Ok(response) => Error::HttpError(response),
            Err(err) => err,
        }
    })?;

    match item.status {
        Status::Pending | Status::InProgress => {
            let response = LambdaResponse::builder()
                .status(StatusCode::ACCEPTED)
                .header("Content-Type", "application/json")
                .body("Running".to_string())
                .map_err(Error::from)?;

            Err(Error::HttpError(response))
        }
        Status::Done(task_result) => {
            let task_result_json = serde_json::to_string(&task_result)?;
            let response = LambdaResponse::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(task_result_json)?;

            Ok(response)
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
            Err(Error::HttpError(val)) => {
                error!("HttpError: {}", val.body());
                Ok(val)
            }
            Err(Error::LambdaError(err)) => {
                error!("LambdaError: {}", err.to_string());
                Err(err)
            }
        }
    }))
    .await
}
