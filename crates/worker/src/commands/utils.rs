// TODO: probably extract preparations to a class

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use std::path::Path;
use std::time::Duration;
use aws_sdk_s3::presigning::PresigningConfig;
use tracing::{error, warn};
use types::item::{Item, Status, TaskResult};
use types::{ARTIFACTS_FOLDER, CompilationRequest};
use uuid::Uuid;

use crate::commands::compile::{
    CompilationArtifact, CompilationInput,
};
use crate::commands::errors::{PreparationError};
use crate::dynamodb_client::DynamoDBClient;
use crate::errors::DBError;
use crate::s3_client::S3Client;
use crate::utils::lib::{SOL_ROOT, ZKSOLC_VERSIONS};

async fn try_set_compiling_status(
    db_client: &DynamoDBClient,
    key: Uuid,
) -> Result<(), PreparationError> {
    let db_update_result = db_client
        .client
        .update_item()
        .table_name(db_client.table_name.clone())
        .key(Item::primary_key_name(), AttributeValue::S(key.to_string()))
        .update_expression("SET #status = :newStatus")
        .condition_expression("#status = :currentStatus")
        .expression_attribute_names("#status", Status::attribute_name())
        .expression_attribute_values(
            ":newStatus",
            AttributeValue::N(u32::from(Status::InProgress).to_string()),
        )
        .expression_attribute_values(
            ":currentStatus",
            AttributeValue::N(u32::from(Status::Pending).to_string()),
        )
        .send()
        .await;

    match db_update_result {
        Ok(_) => Ok(()),
        Err(SdkError::ServiceError(err)) => match err.err() {
            UpdateItemError::ConditionalCheckFailedException(_) => {
                error!("Conditional check not met");
                Err(PreparationError::UnexpectedStatusError(
                    "Concurrent status change from another instance".into(),
                ))
            }
            _ => Err(DBError::from(SdkError::ServiceError(err)).into()),
        },
        Err(err) => Err(DBError::from(err).into()),
    }
}

pub(crate) async fn prepare_compile_input(
    request: &CompilationRequest,
    db_client: &DynamoDBClient,
    s3_client: &S3Client,
) -> Result<CompilationInput, PreparationError> {
    let zksolc_version = request.config.version.as_str();
    if !ZKSOLC_VERSIONS.contains(&zksolc_version) {
        return Err(PreparationError::VersionNotSupported(
            zksolc_version.to_string(),
        ));
    }

    let item = db_client.get_item(request.id.to_string().as_str()).await?;
    let item: Item = match item {
        Some(item) => item,
        None => {
            error!("No item id: {}", request.id);
            return Err(PreparationError::NoDBItemError(request.id.to_string()));
        }
    };

    match item.status {
        Status::Pending => {}
        status => {
            warn!("Item already processing: {}", status);
            return Err(PreparationError::UnexpectedStatusError(status.to_string()));
        }
    }

    let dir = format!("{}/", request.id);
    let files = s3_client.extract_files(&dir).await?;

    // Update status to Compiling
    try_set_compiling_status(db_client, request.id).await?;

    Ok(CompilationInput {
        workspace_path: Path::new(SOL_ROOT).join(request.id.to_string().as_str()),
        config: request.config.clone(),
        contracts: files,
    })
}
pub async fn on_compilation_success(
    id: Uuid,
    db_client: &DynamoDBClient,
    s3_client: &S3Client,
    compilation_artifacts: Vec<CompilationArtifact>,
) -> Result<TaskResult, PreparationError> {
    const DOWNLOAD_URL_EXPIRATION: Duration = Duration::from_secs(5 * 60 * 60);

    let mut presigned_urls = Vec::with_capacity(compilation_artifacts.len());
    for el in compilation_artifacts {
        let file_key = format!("{}/{}/{}", ARTIFACTS_FOLDER, id, el.file_name);
        s3_client.put_object(&file_key, el.file_content).await?;

        let expires_in = PresigningConfig::expires_in(DOWNLOAD_URL_EXPIRATION).unwrap();
        let presigned_request = s3_client
            .get_object_presigned(&file_key, expires_in)
            .await?;

        presigned_urls.push(presigned_request.uri().to_string());
    }

    if presigned_urls.is_empty() {
        // TODO: AttributeValue::Ss doesn't allow empty arrays. Decide what to do. for now
        presigned_urls.push("".to_string());
    }

    db_client
        .client
        .update_item()
        .table_name(db_client.table_name.clone())
        .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
        .update_expression("SET #status = :newStatus, #data = :data")
        .expression_attribute_names("#status", Status::attribute_name())
        .expression_attribute_names("#data", Item::data_attribute_name())
        .expression_attribute_values(
            ":newStatus",
            AttributeValue::N(2.to_string()), // Ready
        )
        .expression_attribute_values(":data", AttributeValue::Ss(presigned_urls.clone()))
        .send()
        .await
        .map_err(DBError::from)?;

    Ok(TaskResult::Success { presigned_urls })
}

pub async fn on_compilation_failed(
    id: Uuid,
    db_client: &DynamoDBClient,
    message: String,
) -> Result<TaskResult, DBError> {
    db_client
        .client
        .update_item()
        .table_name(db_client.table_name.clone())
        .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
        .update_expression("SET #status = :newStatus, #data = :data")
        .expression_attribute_names("#status", Status::attribute_name())
        .expression_attribute_names("#data", Item::data_attribute_name())
        .expression_attribute_values(
            ":newStatus",
            AttributeValue::N(3.to_string()), // Failed
        )
        .expression_attribute_values(":data", AttributeValue::S(message.clone()))
        .send()
        .await?;

    Ok(TaskResult::Failure(message))
}
