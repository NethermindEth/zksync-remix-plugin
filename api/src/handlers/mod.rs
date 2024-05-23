pub mod compile_multiple;
pub mod compiler_version;
pub mod process;
pub mod save_code;
pub mod service_version;
pub mod types;
pub mod verify;

use crate::handlers::compiler_version::do_compiler_version;
use crate::handlers::types::{ApiCommand, ApiCommandResult, HealthCheckResponse};
use crate::handlers::verify::do_verify;
use crate::types::ApiError;
use crate::utils::lib::{get_file_path, init_parent_directories, ARTIFACTS_ROOT};
use rocket::tokio;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;
use tracing::instrument;
use uuid::Uuid;
use crate::handlers::compile_multiple::do_compile;

#[instrument]
#[get("/health")]
pub async fn health() -> HealthCheckResponse {
    info!("/health");

    let file_content = generate_mock_solidity_file_content();
    let version = String::from("latest");
    let path_uuid = generate_remix_file_path();
    let local_path = get_file_path(&version, &path_uuid);
    let path = PathBuf::from_str(&path_uuid).unwrap();

    let artifact_folder = PathBuf::from(ARTIFACTS_ROOT)
        .join(version.clone())
        .join(path_uuid.clone());

    // create file directory from file path
    init_parent_directories(local_path.clone()).await;
    if tokio::fs::write(&local_path, file_content).await.is_err() {
        return HealthCheckResponse::error("Failed to write file");
    }

    let result = do_compile(version, path.clone()).await;

    // cleanup
    if tokio::fs::remove_dir_all(local_path.parent().unwrap().parent().unwrap())
        .await
        .is_err()
    {
        return HealthCheckResponse::error("Failed to remove directory from local path");
    }

    println!("Artifacts : {:?}", artifact_folder);
    if tokio::fs::remove_dir_all(artifact_folder.parent().unwrap())
        .await
        .is_err()
    {
        return HealthCheckResponse::error("Failed to remove directory from artifact path");
    }

    if result.is_ok() {
        HealthCheckResponse::ok()
    } else {
        HealthCheckResponse::error("Failed to compile")
    }
}

pub fn generate_mock_solidity_file_content() -> String {
    r#"
    pragma solidity ^0.8.0;

    contract SimpleStorage {
        uint256 storedData;

        function set(uint256 x) public {
            storedData = x;
        }

        function get() public view returns (uint256) {
            return storedData;
        }
    }
    "#
    .to_string()
}

pub fn generate_remix_file_path() -> String {
    format!("{}/{}", Uuid::new_v4(), "SimpleStorage.sol")
}

#[instrument]
#[get("/")]
pub async fn who_is_this() -> &'static str {
    info!("/who_is_this");
    "Who are you?"
}

pub async fn dispatch_command(command: ApiCommand) -> Result<ApiCommandResult, ApiError> {
    match command {
        ApiCommand::CompilerVersion => match do_compiler_version() {
            Ok(result) => Ok(ApiCommandResult::CompilerVersion(result)),
            Err(e) => Err(e),
        },
        ApiCommand::Compile(request) => match do_compile(request).await {
            Ok(compile_response) => Ok(ApiCommandResult::Compile(compile_response.into_inner())),
            Err(e) => Err(e),
        },
        ApiCommand::Verify {
            path: remix_file_path,
            contract_address,
            network,
            version,
            inputs,
        } => match do_verify(version, network, contract_address, remix_file_path, inputs).await {
            Ok(verify_response) => Ok(ApiCommandResult::Verify(verify_response.into_inner())),
            Err(e) => Err(e),
        },
        ApiCommand::Shutdown => Ok(ApiCommandResult::Shutdown),
    }
}
