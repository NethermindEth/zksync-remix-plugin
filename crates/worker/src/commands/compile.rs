use crate::commands::SPAWN_SEMAPHORE;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::presigning::PresigningConfig;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tracing::warn;
use tracing::{error, info};
use types::item::{Item, Status};
use types::{CompilationConfig, CompilationRequest, ARTIFACTS_FOLDER};

use crate::dynamodb_client::DynamoDBClient;
use crate::errors::{CompilationError, DBError};
use crate::s3_client::S3Client;
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    initialize_files, list_files_in_directory, DEFAULT_SOLIDITY_VERSION, SOL_ROOT, ZKSOLC_VERSIONS,
};

pub struct CompilationFile {
    pub file_path: String,
    pub file_content: Vec<u8>,
}

pub struct CompilationInput {
    pub config: CompilationConfig,
    // legacy. files really
    pub contracts: Vec<CompilationFile>,
}

pub struct CompilationArtifact {
    pub file_name: String,
    pub file_content: Vec<u8>,
    pub is_contract: bool,
}

pub async fn compile(
    request: CompilationRequest,
    db_client: &DynamoDBClient,
    s3_client: &S3Client,
) -> Result<(), CompilationError> {
    let item = db_client.get_item(request.id.clone()).await?;
    let item: Item = match item {
        Some(item) => item,
        None => {
            error!("No item id: {}", request.id);
            return Err(CompilationError::NoDBItemError(request.id));
        }
    };

    match item.status {
        Status::Pending => {}
        status => {
            warn!("Item already processing: {}", status);
            return Err(CompilationError::UnexpectedStatusError(status.to_string()));
        }
    }

    let dir = format!("{}/", request.id);
    let files = s3_client.extract_files(&dir).await?;
    {
        let db_update_result = db_client
            .client
            .update_item()
            .table_name(db_client.table_name.clone())
            .key("ID", AttributeValue::S(request.id.clone()))
            .update_expression("SET #status = :newStatus")
            .condition_expression("#status = :currentStatus")
            .expression_attribute_names("#status", Status::db_key_name())
            .expression_attribute_values(
                ":newStatus",
                AttributeValue::N(u32::from(Status::Compiling).to_string()),
            )
            .expression_attribute_values(
                ":currentStatus",
                AttributeValue::N(u32::from(Status::Pending).to_string()),
            )
            .send()
            .await;
        match db_update_result {
            Ok(_) => {}
            Err(SdkError::ServiceError(err)) => {
                return match err.err() {
                    UpdateItemError::ConditionalCheckFailedException(_) => {
                        error!("Conditional check not met");
                        Err(CompilationError::UnexpectedStatusError(
                            "Concurrent status change from another instance".into(),
                        ))
                    }
                    _ => Err(DBError::from(SdkError::ServiceError(err)).into()),
                }
            }
            Err(err) => return Err(DBError::from(err).into()),
        }
    }

    match do_compile(
        &request.id,
        CompilationInput {
            config: request.config,
            contracts: files,
        },
    )
    .await
    {
        Ok(value) => Ok(on_compilation_success(&request.id, db_client, s3_client, value).await?),
        Err(err) => match err {
            CompilationError::CompilationFailureError(value) => {
                Ok(on_compilation_failed(&request.id, db_client, value).await?)
            }
            CompilationError::VersionNotSupported(value) => Ok(on_compilation_failed(
                &request.id,
                &db_client,
                format!("Unsupported compiler version: {}", value),
            )
            .await?),
            _ => Err(err),
        },
    }
}

pub async fn on_compilation_success(
    id: &str,
    db_client: &DynamoDBClient,
    s3_client: &S3Client,
    compilation_artifacts: Vec<CompilationArtifact>,
) -> Result<(), CompilationError> {
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
        .key("ID", AttributeValue::S(id.to_string()))
        .update_expression("SET #status = :newStatus, #data = :data")
        .expression_attribute_names("#status", Status::db_key_name())
        .expression_attribute_names("#data", "Data")
        .expression_attribute_values(
            ":newStatus",
            AttributeValue::N(2.to_string()), // Ready
        )
        .expression_attribute_values(":data", AttributeValue::Ss(presigned_urls))
        .send()
        .await
        .map_err(DBError::from)?;

    Ok(())
}

pub async fn on_compilation_failed(
    id: &str,
    db_client: &DynamoDBClient,
    message: String,
) -> Result<(), DBError> {
    db_client
        .client
        .update_item()
        .table_name(db_client.table_name.clone())
        .key("ID", AttributeValue::S(id.to_string()))
        .update_expression("SET #status = :newStatus, #data = :data")
        .expression_attribute_names("#status", Status::db_key_name())
        .expression_attribute_names("#data", "Data")
        .expression_attribute_values(
            ":newStatus",
            AttributeValue::N(3.to_string()), // Failed
        )
        .expression_attribute_values(":data", AttributeValue::S(message))
        .send()
        .await?;

    Ok(())
}

pub async fn do_compile(
    namespace: &str,
    compilation_request: CompilationInput,
) -> Result<Vec<CompilationArtifact>, CompilationError> {
    let zksolc_version = compilation_request.config.version;

    // check if the version is supported
    if !ZKSOLC_VERSIONS.contains(&zksolc_version.as_str()) {
        return Err(CompilationError::VersionNotSupported(zksolc_version));
    }

    // root directory for the contracts
    let workspace_path = Path::new(SOL_ROOT).join(namespace);
    // root directory for the artifacts
    let artifacts_path = workspace_path.join("artifacts-zk");
    // root directory for user files (hardhat config, etc)
    let hardhat_config_path = workspace_path.join("hardhat.config.ts");

    // instantly create the directories
    tokio::fs::create_dir_all(&workspace_path).await?;
    tokio::fs::create_dir_all(&artifacts_path).await?;

    // when the compilation is done, clean up the directories
    // it will be called when the AutoCleanUp struct is dropped
    let auto_clean_up = AutoCleanUp {
        dirs: vec![workspace_path.to_str().unwrap()],
    };

    // write the hardhat config file
    let mut hardhat_config_builder = HardhatConfigBuilder::new();
    hardhat_config_builder
        .zksolc_version(&zksolc_version)
        .solidity_version(DEFAULT_SOLIDITY_VERSION);
    if let Some(target_path) = compilation_request.config.target_path {
        hardhat_config_builder.paths_sources(&target_path);
    }

    let hardhat_config_content = hardhat_config_builder.build().to_string_config();

    // create parent directories
    tokio::fs::create_dir_all(hardhat_config_path.parent().unwrap()).await?;
    tokio::fs::write(hardhat_config_path, hardhat_config_content).await?;

    // filter test files from compilation candidates
    let contracts = compilation_request
        .contracts
        .into_iter()
        .filter(|contract| !contract.file_path.ends_with("_test.sol"))
        .collect();

    // initialize the files
    initialize_files(&workspace_path, contracts).await?;

    // Limit number of spawned processes. RAII released
    let _permit = SPAWN_SEMAPHORE.acquire().await.expect("Expired semaphore");

    let process = tokio::process::Command::new("npx")
        .arg("hardhat")
        .arg("compile")
        .current_dir(&workspace_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = process.wait_with_output().await?;

    let status = output.status;
    let message = String::from_utf8_lossy(&output.stdout).to_string();
    info!("Output: \n{:?}", message);

    if !status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        error!("Compilation error: {}", err_msg);
        return Err(CompilationError::CompilationFailureError(
            err_msg.to_string(),
        ));
    }

    // fetch the files in the artifacts directory
    let mut file_contents: Vec<CompilationArtifact> = vec![];
    let file_paths =
        list_files_in_directory(&artifacts_path).expect("Unexpected error listing artifact");
    for file_path in file_paths.iter() {
        // TODO: change this - don't store files in RAM. copy 1-1 to S3
        let file_content = tokio::fs::read(file_path).await?;
        let full_path = Path::new(file_path);

        let relative_path = full_path
            .strip_prefix(&artifacts_path)
            .expect("Unexpected prefix");
        let relative_path_str = relative_path.to_str().unwrap();

        let is_contract =
            !relative_path_str.ends_with(".dbg.json") && relative_path_str.ends_with(".json");

        file_contents.push(CompilationArtifact {
            file_name: relative_path_str.to_string(),
            file_content,
            is_contract,
        });
    }

    // calling here explicitly to avoid dropping the AutoCleanUp struct
    auto_clean_up.clean_up().await;
    Ok(file_contents)
}
