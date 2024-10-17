use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_http::http::StatusCode;
use lambda_http::tower::util::Either;
use lambda_http::{
    run, service_fn, Error as LambdaError, Request as LambdaRequest, Response as LambdaResponse,
};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info, warn};
use types::item::errors::ItemError;
use types::item::{Item, Status};
use uuid::Uuid;

const TABLE_NAME_DEFAULT: &str = "zksync-table";
const NO_SUCH_ITEM: &str = "No such item";

mod common;
use crate::common::utils::error_string_to_json;

#[derive(Deserialize)]
struct PollRequest {
    pub id: Uuid,
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("{0}")]
    BadRequestError(String),
    #[error("{0}")]
    NotFoundError(String),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
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
            Self::NotFoundError(err) => {
                let response = LambdaResponse::builder()
                    .status(StatusCode::NOT_FOUND)
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

#[tracing::instrument(skip(request))]
fn extract_uuid_from_request(request: &LambdaRequest) -> Result<Uuid, Error> {
    let path = request.uri().path();
    let path_segments: Vec<&str> = path.split('/').collect();

    let uuid_str = path_segments.last().ok_or_else(|| {
        info!("Invalid user request: {}", path);
        Error::BadRequestError(format!("Invalid url: {}", path))
    })?;
    let uuid = Uuid::parse_str(uuid_str).map_err(|err| {
        info!("User supplied invalid uuid: {}", uuid_str);
        Error::BadRequestError(format!("Invalid UUID: {}", err.to_string()))
    })?;

    Ok(uuid)
}

// Returns either Ok or Accepted
#[tracing::instrument(skip(dynamo_client, table_name))]
async fn process_request(
    request: LambdaRequest,
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
) -> Result<Either<String, ()>, Error> {
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
        .map_err(anyhow::Error::from)?;

    let raw_item = output.item.ok_or_else(|| {
        info!("Requested non existing item: {}", request.id);
        Error::NotFoundError(NO_SUCH_ITEM.to_string())
    })?;

    let item: Item = raw_item.try_into().map_err(|err: ItemError| {
        error!("Failed to deserialize item. id: {}", request.id);
        Error::InternalError(err.into())
    })?;

    match item.status {
        Status::Pending | Status::InProgress => Ok(Either::B(())),
        Status::Done(task_result) => {
            let task_result_json =
                serde_json::to_string(&task_result).map_err(anyhow::Error::from)?;
            Ok(Either::A(task_result_json))
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
            Ok(Either::A(ok_json)) => {
                let response = LambdaResponse::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(ok_json)?;

                Ok(response)
            }
            Ok(Either::B(_)) => {
                let response = LambdaResponse::builder()
                    .status(StatusCode::ACCEPTED)
                    .header("Content-Type", "application/json")
                    .body(json!({
                        "status": "Running"
                    }).to_string())?;

                Ok(response)
            }
            Err(err) => Ok::<_, LambdaError>(err.try_into()?),
        }
    }))
    .await
}
