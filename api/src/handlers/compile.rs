use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{
    ApiCommand, ApiCommandResult, CompilationRequest, CompileResponse, CompiledFile,
};
use crate::rate_limiter::RateLimited;
use crate::types::{ApiError, Result};
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    generate_folder_name, initialize_files, list_files_in_directory, status_code_to_message,
    ARTIFACTS_ROOT, DEFAULT_SOLIDITY_VERSION, HARDHAT_CACHE_PATH, HARDHAT_ENV_DOCKER_IMAGE,
    HARDHAT_ENV_ROOT, SOL_ROOT, ZKSOLC_VERSIONS,
};
use crate::worker::WorkerEngine;
use rocket::serde::json;
use rocket::serde::json::Json;
use rocket::{tokio, State};
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::info;
use tracing::instrument;

#[instrument]
#[post("/compile", format = "json", data = "<request_json>")]
pub async fn compile(
    request_json: Json<CompilationRequest>,
    _rate_limited: RateLimited,
) -> Json<CompileResponse> {
    info!("/compile/{:?}", request_json.config);

    do_compile(request_json.0).await.unwrap_or_else(|e| {
        Json(CompileResponse {
            file_content: vec![],
            message: e.to_string(),
            status: "Error".to_string(),
        })
    })
}

#[instrument]
#[post("/compile-async", format = "json", data = "<request_json>")]
pub async fn compile_async(
    request_json: Json<CompilationRequest>,
    _rate_limited: RateLimited,
    engine: &State<WorkerEngine>,
) -> String {
    info!("/compile-async/{:?}", request_json.config);

    do_process_command(ApiCommand::Compile(request_json.0), engine)
}

#[instrument]
#[get("/compile-result/<process_id>")]
pub async fn get_compile_result(process_id: String, engine: &State<WorkerEngine>) -> String {
    info!("/compile-result/{:?}", process_id);

    fetch_process_result(process_id, engine, |result| match result {
        ApiCommandResult::Compile(compilation_result) => {
            json::to_string(&compilation_result).unwrap_or_default()
        }
        _ => String::from("Result not available"),
    })
}

pub async fn do_compile(compilation_request: CompilationRequest) -> Result<Json<CompileResponse>> {
    let zksolc_version = compilation_request.config.version;

    // check if the version is supported
    if !ZKSOLC_VERSIONS.contains(&zksolc_version.as_str()) {
        return Err(ApiError::VersionNotSupported(zksolc_version));
    }

    let namespace = generate_folder_name();

    // root directory for the contracts
    let contracts_path_str = format!("{}/{}", SOL_ROOT, namespace);
    let contracts_path = Path::new(&contracts_path_str);

    // root directory for the artifacts
    let artifacts_path_str = format!("{}/{}", ARTIFACTS_ROOT, namespace);
    let artifacts_path = Path::new(&artifacts_path_str);

    // root directory for user files (hardhat config, etc)
    let user_files_path_str = format!("{}/user-{}", HARDHAT_ENV_ROOT, namespace);
    let hardhat_config_path = Path::new(&user_files_path_str).join("hardhat.config.ts");

    // instantly create the directories
    tokio::fs::create_dir_all(contracts_path)
        .await
        .map_err(ApiError::FailedToWriteFile)?;
    tokio::fs::create_dir_all(artifacts_path)
        .await
        .map_err(ApiError::FailedToWriteFile)?;

    // when the compilation is done, clean up the directories
    // it will be called when the AutoCleanUp struct is dropped
    let auto_clean_up = AutoCleanUp {
        dirs: vec![
            contracts_path.to_str().unwrap(),
            artifacts_path.to_str().unwrap(),
            &user_files_path_str,
        ],
    };

    // write the hardhat config file
    let hardhat_config_content = HardhatConfigBuilder::new()
        .zksolc_version(&zksolc_version)
        .solidity_version(DEFAULT_SOLIDITY_VERSION)
        .build()
        .to_string_config();

    // create parent directories
    tokio::fs::create_dir_all(hardhat_config_path.parent().unwrap())
        .await
        .map_err(ApiError::FailedToWriteFile)?;

    tokio::fs::write(hardhat_config_path, hardhat_config_content)
        .await
        .map_err(ApiError::FailedToWriteFile)?;

    // initialize the files
    initialize_files(compilation_request.contracts, contracts_path).await?;

    let command = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .args([
            "-v",
            &format!("{}:/app/contracts", contracts_path.to_str().unwrap()),
        ])
        .args([
            "-v",
            &format!("{}:/app/artifacts-zk", artifacts_path.to_str().unwrap()),
        ])
        .args([
            "-v",
            &format!("{}:/root/.cache/hardhat-nodejs/", HARDHAT_CACHE_PATH),
        ])
        .args([
            "-v",
            &format!(
                "{}/hardhat.config.ts:/app/hardhat.config.ts",
                user_files_path_str
            ),
        ])
        .arg(HARDHAT_ENV_DOCKER_IMAGE)
        .args(["npx", "hardhat", "compile"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let process = command.map_err(ApiError::FailedToExecuteCommand)?;
    let output = process
        .wait_with_output()
        .map_err(ApiError::FailedToReadOutput)?;
    let status = output.status;
    let message = String::from_utf8_lossy(&output.stdout).to_string();

    if !status.success() {
        return Ok(Json(CompileResponse {
            file_content: vec![],
            message: format!(
                "Failed to compile:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ),
            status: "Error".to_string(),
        }));
    }

    // fetch the files in the artifacts directory
    let mut file_contents: Vec<CompiledFile> = vec![];
    let file_paths = list_files_in_directory(artifacts_path);

    for file_path in file_paths.iter() {
        let file_content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(ApiError::FailedToReadFile)?;
        let full_path = Path::new(file_path);
        let relative_path = full_path.strip_prefix(artifacts_path).unwrap_or(full_path);
        let relative_path_str = relative_path.to_str().unwrap();

        // todo(varex83): is it the best way to check?
        let is_contract = relative_path_str.starts_with("contracts")
            && !relative_path_str.ends_with(".dbg.json")
            && relative_path_str.ends_with(".json");

        file_contents.push(CompiledFile {
            file_name: relative_path_str.to_string(),
            file_content,
            is_contract,
        });
    }

    // calling here explicitly to avoid dropping the AutoCleanUp struct
    auto_clean_up.clean_up().await;

    Ok(Json(CompileResponse {
        file_content: file_contents,
        status: status_code_to_message(status.code()),
        message,
    }))
}
