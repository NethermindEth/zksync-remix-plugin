use crate::commands::SPAWN_SEMAPHORE;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::Object;
use std::ops::Add;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tracing::warn;
use tracing::{error, info};
use types::item::{Item, Status};
use types::{CompilationConfig, CompilationRequest, ARTIFACTS_FOLDER};

use crate::dynamodb_client::DynamoDBClient;
use crate::errors::{CompilationError, DBError, S3Error};
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    initialize_files, list_files_in_directory,
    DEFAULT_SOLIDITY_VERSION, SOL_ROOT, ZKSOLC_VERSIONS,
};

pub struct CompilationFile {
    // legacy name. file_path really
    pub file_name: String,
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
    s3_client: &aws_sdk_s3::Client,
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

    let files = extract_files(request.id.clone(), s3_client).await?;
    {
        let db_update_result = db_client
            .client
            .update_item()
            .table_name(db_client.table_name.clone())
            .key("ID", AttributeValue::S(request.id.clone()))
            .update_expression("SET Status = :newStatus")
            .condition_expression("Status = :currentStatus") // TODO: check
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
            Err(SdkError::ServiceError(err)) => match err.err() {
                UpdateItemError::ConditionalCheckFailedException(_) => {
                    return Err(CompilationError::UnexpectedStatusError(
                        "Concurrent status change from another instance".into(),
                    ))
                }
                _ => return Err(DBError::from(SdkError::ServiceError(err)).into()),
            },
            Err(err) => return Err(DBError::from(err).into()),
        }
    }

    match do_compile(
        request.id.clone(),
        CompilationInput {
            config: request.config,
            contracts: files,
        },
    )
        .await
    {
        Ok(val) => Ok(on_compilation_success(&request.id, db_client, s3_client, val).await?),
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

async fn list_all_keys(
    client: &aws_sdk_s3::Client,
    id: String,
    bucket: &str,
) -> Result<Vec<Object>, S3Error> {
    let mut objects = Vec::new();
    let mut continuation_token: Option<String> = None;

    let id = id.clone().add("/");
    loop {
        let mut request = client
            .list_objects_v2()
            .bucket(bucket)
            .delimiter('/')
            .prefix(id.clone());
        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }

        let response = request.send().await?;
        if let Some(contents) = response.contents {
            objects.extend(contents);
        }

        let is_truncated = if let Some(is_truncated) = response.is_truncated {
            is_truncated
        } else {
            warn!("is_truncated empty");
            break;
        };

        if !is_truncated {
            break;
        }

        continuation_token = response.next_continuation_token;
        if continuation_token.is_none() {
            error!("continuation_token wasn't set!");
            break;
        }
    }

    Ok(objects)
}

async fn extract_files(
    id: String,
    s3_client: &aws_sdk_s3::Client,
) -> Result<Vec<CompilationFile>, S3Error> {
    let objects = list_all_keys(&s3_client, id.to_string(), "TODO").await?;

    let mut files = vec![];
    for object in objects {
        let key = object.key().ok_or(S3Error::InvalidObjectError)?;
        let expected_size = object.size.ok_or(S3Error::InvalidObjectError)?;

        let mut object = s3_client
            .get_object()
            .bucket("TODO:")
            .key(key)
            .send()
            .await?;

        let mut byte_count = 0;
        let mut contents = Vec::new();
        while let Some(bytes) = object.body.try_next().await? {
            let bytes_len = bytes.len();
            std::io::Write::write_all(&mut contents, &bytes)?;
            byte_count += bytes_len;
        }

        if byte_count as i64 != expected_size {
            error!("Fetched num bytes != expected size of file.");
            return Err(S3Error::InvalidObjectError);
        }

        files.push(CompilationFile {
            file_content: contents,
            file_name: key.to_string(),
        });
    }

    Ok(files)
}

pub async fn on_compilation_success(
    id: &str,
    db_client: &DynamoDBClient,
    s3_client: &aws_sdk_s3::Client,
    compilation_artifacts: Vec<CompilationArtifact>,
) -> Result<(), CompilationError> {
    let mut presigned_urls = Vec::with_capacity(compilation_artifacts.len());
    for el in compilation_artifacts {
        let file_key = format!("{}/{}/{}", ARTIFACTS_FOLDER, id, el.file_name);
        s3_client
            .put_object()
            .bucket("TODO")
            .key(file_key.clone())
            .body(el.file_content.into())
            .send()
            .await
            .map_err(S3Error::from)?;

        let expires_in = PresigningConfig::expires_in(Duration::from_secs(5 * 60 * 60)).unwrap();
        let presigned_request = s3_client
            .get_object()
            .bucket("TODO")
            .key(file_key)
            .presigned(expires_in)
            .await
            .map_err(S3Error::from)?;

        presigned_urls.push(presigned_request.uri().to_string());
    }

    db_client
        .client
        .update_item()
        .table_name(db_client.table_name.clone())
        .key("ID", AttributeValue::S(id.to_string()))
        .update_expression("SET Status = :newStatus")
        .update_expression("SET Data = :data")
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
        .update_expression("SET Status = :newStatus")
        .update_expression("SET Data = :data")
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
    namespace: String,
    compilation_request: CompilationInput,
) -> Result<Vec<CompilationArtifact>, CompilationError> {
    let zksolc_version = compilation_request.config.version;

    // check if the version is supported
    if !ZKSOLC_VERSIONS.contains(&zksolc_version.as_str()) {
        return Err(CompilationError::VersionNotSupported(zksolc_version));
    }

    // root directory for the contracts
    let workspace_path_str = format!("{}/{}", SOL_ROOT, namespace);
    let workspace_path = Path::new(&workspace_path_str);

    // root directory for the artifacts
    let artifacts_path_str = format!("{}/{}", workspace_path_str, "artifacts-zk");
    let artifacts_path = Path::new(&artifacts_path_str);

    // root directory for user files (hardhat config, etc)
    let user_files_path_str = workspace_path_str.clone();
    let hardhat_config_path = Path::new(&user_files_path_str).join("hardhat.config.ts");

    // instantly create the directories
    tokio::fs::create_dir_all(workspace_path).await?;
    tokio::fs::create_dir_all(artifacts_path).await?;

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
        .filter(|contract| !contract.file_name.ends_with("_test.sol"))
        .collect();

    // initialize the files
    initialize_files(workspace_path, contracts).await?;

    // Limit number of spawned processes. RAII released
    let _permit = SPAWN_SEMAPHORE.acquire().await.expect("Expired semaphore");

    let process = tokio::process::Command::new("npx")
        .arg("hardhat")
        .arg("compile")
        .current_dir(workspace_path)
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
        // TODO: handle
        return Err(CompilationError::CompilationFailureError(err_msg.to_string()));
    }

    // fetch the files in the artifacts directory
    let mut file_contents: Vec<CompilationArtifact> = vec![];
    let file_paths =
        list_files_in_directory(artifacts_path).expect("Unexpected error listing artifact");
    for file_path in file_paths.iter() {
        // TODO: change this - don't store files in RAM. copy 1-1 to S3
        let file_content = tokio::fs::read(file_path).await?;
        let full_path = Path::new(file_path);

        let relative_path = full_path
            .strip_prefix(artifacts_path)
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
