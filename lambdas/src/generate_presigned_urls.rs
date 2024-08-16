use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use lambda_http::http::StatusCode;
use lambda_http::{
    run, service_fn, Error as LambdaError, IntoResponse, Request as LambdaRequest,
    RequestPayloadExt, Response as LambdaResponse,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{error, Value};
use uuid::Uuid;

mod common;
use common::BUCKET_NAME_DEFAULT;

const MAX_FILES: usize = 300;
const OBJECT_EXPIRATION_TIME: Duration = Duration::from_secs(24 * 60 * 60);

const PAYLOAD_EMPTY_ERROR: &str = "Request payload is empty";
const EXCEEDED_MAX_FILES_ERROR: &str = "Exceeded max number of files";

#[derive(Debug, Deserialize)]
struct Request {
    pub files: Vec<PathBuf>,
    // TODO: add files_md5. Use set_content_md5 on object
}

#[derive(Debug, Serialize)]
struct Response {
    pub id: Uuid,
    pub presigned_urls: Vec<String>,
}

async fn generate_presigned_urs(
    files: Vec<PathBuf>,
    bucket_name: &str,
    s3_client: &aws_sdk_s3::Client,
) -> Result<Response, LambdaError> {
    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    let uuid_dir = Path::new(&uuid_str);

    let mut output = Vec::with_capacity(files.len());
    for file in files {
        let presigned = s3_client
            .put_object()
            .bucket(bucket_name)
            .key(uuid_dir.join(file).to_string_lossy().to_string())
            .presigned(PresigningConfig::expires_in(OBJECT_EXPIRATION_TIME).map_err(Box::new)?)
            .await?;

        output.push(presigned.uri().into());
    }

    Ok(Response {
        id: uuid,
        presigned_urls: output,
    })
}

#[tracing::instrument]
async fn process_request(
    request: LambdaRequest,
    bucket_name: &str,
    s3_client: &aws_sdk_s3::Client,
) -> Result<LambdaResponse<String>, LambdaError> {
    let request = match request.payload::<Request>() {
        Ok(Some(val)) => val,
        Ok(None) => {
            let response = LambdaResponse::builder()
                .status(400)
                .header("content-type", "text/html")
                .body("Request payload is empty".into())
                .map_err(Box::new)?;

            return Ok(response);
        }
        Err(err) => {
            let response = LambdaResponse::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(err.to_string().into())
                .map_err(Box::new)?;

            return Ok(response);
        }
    };

    if request.files.len() > MAX_FILES {
        let response = LambdaResponse::builder()
            .status(400)
            .header("content-type", "text/html")
            .body(EXCEEDED_MAX_FILES_ERROR.into())
            .map_err(Box::new)?;

        return Ok(response);
    }

    let response = generate_presigned_urs(request.files, bucket_name, s3_client).await?;
    let response = LambdaResponse::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&response)?)?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .without_time() // CloudWatch will add the ingestion time
        .with_target(false)
        .init();

    let bucket_name = std::env::var("BUCKET_NAME").unwrap_or(BUCKET_NAME_DEFAULT.into());
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&aws_config);

    run(service_fn(|request: LambdaRequest| async {
        process_request(request, &bucket_name, &s3_client).await
    }))
    .await
}
