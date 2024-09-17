// TODO: probably extract preparations to a class

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::presigning::PresigningConfig;
use std::path::Path;
use std::time::Duration;
use tracing::{error, warn};
use types::item::{Item, Status, TaskResult};
use types::{CompilationRequest, ARTIFACTS_FOLDER};
use uuid::Uuid;

use crate::clients::dynamodb_clients::wrapper::DynamoDBClientWrapper;
use crate::clients::errors::{DBError, S3Error};
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::commands::compile::{CompilationInput, CompilationOutput};
use crate::commands::errors::{CommandResultHandleError, PreparationError};
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::lib::{SOL_ROOT, ZKSOLC_VERSIONS};

async fn try_set_compiling_status(
    db_client: &DynamoDBClientWrapper,
    key: Uuid,
) -> Result<(), PreparationError> {
    let db_update_result = db_client
        .update_item_status_conditional(
            key.to_string().as_str(),
            &Status::Pending,
            &Status::InProgress,
        )
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
    db_client: &DynamoDBClientWrapper,
    s3_client: &S3ClientWrapper,
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
            return Err(PreparationError::NoDBItemError(request.id.to_string()));
        }
    };

    if !matches!(item.status, Status::Pending) {
        warn!("Item already processing: {}", item.status);
        return Err(PreparationError::UnexpectedStatusError(item.status.to_string()));
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
    db_client: &DynamoDBClientWrapper,
    s3_client: &S3ClientWrapper,
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
        s3_client.put_object(&file_key, file_content).await?;

        let expires_in = PresigningConfig::expires_in(DOWNLOAD_URL_EXPIRATION).unwrap();
        let presigned_request = s3_client
            .get_object_presigned(&file_key, &expires_in)
            .await
            .map_err(S3Error::from)?;

        presigned_urls.push(presigned_request.uri().to_string());
    }

    if presigned_urls.is_empty() {
        // TODO: AttributeValue::Ss doesn't allow empty arrays. Decide what to do. for now
        presigned_urls.push("".to_string());
    }

    let builder = db_client
        .client
        .client
        .update_item()
        .table_name(db_client.client.table_name.clone())
        .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
        .update_expression("SET #status = :newStatus, #data = :data")
        .expression_attribute_names("#status", Status::attribute_name())
        .expression_attribute_names("#data", Item::data_attribute_name())
        .expression_attribute_values(
            ":newStatus",
            AttributeValue::N(2.to_string()), // Ready
        )
        .expression_attribute_values(":data", AttributeValue::Ss(presigned_urls.clone()));

    db_client.update_item_raw(builder).await.map_err(DBError::from)?;

    auto_clean_up.clean_up().await;
    Ok(TaskResult::Success { presigned_urls })
}

pub async fn on_compilation_failed(
    id: Uuid,
    db_client: &DynamoDBClientWrapper,
    message: String,
) -> Result<TaskResult, CommandResultHandleError> {
    let builder = db_client
        .client
        .client
        .update_item()
        .table_name(db_client.client.table_name.clone())
        .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
        .update_expression("SET #status = :newStatus, #data = :data")
        .expression_attribute_names("#status", Status::attribute_name())
        .expression_attribute_names("#data", Item::data_attribute_name())
        .expression_attribute_values(
            ":newStatus",
            AttributeValue::N(3.to_string()), // Failed
        )
        .expression_attribute_values(":data", AttributeValue::S(message.clone()));

    db_client.update_item_raw(builder).await.map_err(DBError::from)?;

    Ok(TaskResult::Failure(message))
}
