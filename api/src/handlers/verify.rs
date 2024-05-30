use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult, VerificationRequest, VerifyResponse};
use crate::rate_limiter::RateLimited;
use crate::types::{ApiError, Result};
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    generate_folder_name, initialize_files, ALLOWED_NETWORKS, ARTIFACTS_ROOT,
    DEFAULT_SOLIDITY_VERSION, HARDHAT_CACHE_PATH, HARDHAT_ENV_DOCKER_IMAGE, HARDHAT_ENV_ROOT,
    SOL_ROOT, ZKSOLC_VERSIONS,
};
use crate::worker::WorkerEngine;
use rocket::serde::{json, json::Json};
use rocket::{tokio, State};
use std::path::Path;
use std::process::{Command, Stdio};
use tracing::info;
use tracing::instrument;

#[instrument]
#[post("/verify", format = "json", data = "<verification_request_json>")]
pub async fn verify(
    verification_request_json: Json<VerificationRequest>,
    _rate_limited: RateLimited,
) -> Json<VerifyResponse> {
    info!("/verify");

    do_verify(verification_request_json.0)
        .await
        .unwrap_or_else(|e| {
            Json(VerifyResponse {
                message: e.to_string(),
                status: "Error".to_string(),
            })
        })
}

#[instrument]
#[post("/verify-async", format = "json", data = "<verification_request_json>")]
pub fn verify_async(
    verification_request_json: Json<VerificationRequest>,
    _rate_limited: RateLimited,
    engine: &State<WorkerEngine>,
) -> String {
    info!("/verify-async",);

    do_process_command(ApiCommand::Verify(verification_request_json.0), engine)
}

#[instrument]
#[get("/verify-result/<process_id>")]
pub async fn get_verify_result(process_id: String, engine: &State<WorkerEngine>) -> String {
    info!("/verify-result/{:?}", process_id);

    fetch_process_result(process_id, engine, |result| match result {
        ApiCommandResult::Verify(verification_result) => {
            json::to_string(&verification_result).unwrap_or_default()
        }
        _ => String::from("Result not available"),
    })
}

pub async fn do_verify(verification_request: VerificationRequest) -> Result<Json<VerifyResponse>> {
    let zksolc_version = verification_request.config.zksolc_version;

    // check if the version is supported
    if !ZKSOLC_VERSIONS.contains(&zksolc_version.as_str()) {
        return Err(ApiError::VersionNotSupported(zksolc_version));
    }

    let solc_version = verification_request
        .config
        .solc_version
        .unwrap_or(DEFAULT_SOLIDITY_VERSION.to_string());

    let network = verification_request.config.network;

    // check if the network is supported
    if !ALLOWED_NETWORKS.contains(&network.as_str()) {
        return Err(ApiError::UnknownNetwork(network));
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
        .solidity_version(&solc_version)
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
    initialize_files(verification_request.contracts.clone(), contracts_path).await?;

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
        .args(["npx", "hardhat", "verify"])
        .args([
            "--network",
            if network == "sepolia" {
                "zkSyncTestnet"
            } else {
                "zkSyncMainnet"
            },
        ])
        .arg(verification_request.config.contract_address)
        .args(verification_request.config.inputs)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let process = command.map_err(ApiError::FailedToExecuteCommand)?;
    let output = process
        .wait_with_output()
        .map_err(ApiError::FailedToReadOutput)?;
    let status = output.status;
    let message = String::from_utf8_lossy(&output.stdout).to_string();

    info!("Output: \n{:?}", String::from_utf8_lossy(&output.stdout));
    if !status.success() {
        return Ok(Json(VerifyResponse {
            status: "Error".to_string(),
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        }));
    }

    // calling here explicitly to avoid dropping the AutoCleanUp struct
    auto_clean_up.clean_up().await;

    Ok(Json(VerifyResponse {
        status: "Success".to_string(),
        message,
    }))
}
