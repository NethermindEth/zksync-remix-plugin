use rocket::serde::{json, json::Json};
use rocket::{tokio, State};
use std::path::Path;
use std::process::Stdio;
use tracing::info;
use tracing::instrument;

use crate::errors::{ApiError, Result};
use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult, VerificationRequest, VerifyResponse};
use crate::handlers::SPAWN_SEMAPHORE;
use crate::metrics::Metrics;
use crate::rate_limiter::RateLimited;
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    generate_folder_name, initialize_files, ALLOWED_NETWORKS, DEFAULT_SOLIDITY_VERSION, SOL_ROOT,
    ZKSOLC_VERSIONS,
};
use crate::worker::WorkerEngine;

pub(crate) const VERIFICATION_LABEL_VALUE: &str = "compilation";

#[instrument(skip(verification_request_json, _rate_limited, engine))]
#[post("/verify", format = "json", data = "<verification_request_json>")]
pub async fn verify(
    verification_request_json: Json<VerificationRequest>,
    _rate_limited: RateLimited,
    engine: &State<WorkerEngine>,
) -> Json<VerifyResponse> {
    info!("/verify/{:?}", verification_request_json.config);

    do_verify(verification_request_json.0, &engine.metrics)
        .await
        .unwrap_or_else(|e| {
            Json(VerifyResponse {
                message: e.to_string(),
                status: "Error".to_string(),
            })
        })
}

#[instrument(skip(verification_request_json, _rate_limited, engine))]
#[post("/verify-async", format = "json", data = "<verification_request_json>")]
pub fn verify_async(
    verification_request_json: Json<VerificationRequest>,
    _rate_limited: RateLimited,
    engine: &State<WorkerEngine>,
) -> String {
    info!("/verify-async/{:?}", verification_request_json.config);

    do_process_command(ApiCommand::Verify(verification_request_json.0), engine)
}

#[instrument(skip(engine))]
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

fn extract_verify_args(request: &VerificationRequest) -> Vec<String> {
    let mut args: Vec<String> = vec!["hardhat".into(), "verify".into(), "--network".into()];
    if request.config.network == "sepolia" {
        args.push("zkSyncTestnet".into())
    } else {
        args.push("zkSyncMainnet".into())
    }

    if let Some(ref target_contract) = request.target_contract {
        args.push("--contract".into());
        args.push(target_contract.clone());
    }

    args.push(request.config.contract_address.clone());
    args.extend(request.config.inputs.clone());

    args
}

pub async fn do_verify(
    verification_request: VerificationRequest,
    metrics: &Metrics,
) -> Result<Json<VerifyResponse>> {
    let zksolc_version = verification_request.config.zksolc_version.clone();

    // check if the version is supported
    if !ZKSOLC_VERSIONS.contains(&zksolc_version.as_str()) {
        return Err(ApiError::VersionNotSupported(zksolc_version));
    }

    let solc_version = verification_request
        .config
        .solc_version
        .clone()
        .unwrap_or(DEFAULT_SOLIDITY_VERSION.to_string());

    let network = verification_request.config.network.clone();

    // check if the network is supported
    if !ALLOWED_NETWORKS.contains(&network.as_str()) {
        return Err(ApiError::UnknownNetwork(network));
    }

    let namespace = generate_folder_name();

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
    tokio::fs::create_dir_all(workspace_path)
        .await
        .map_err(ApiError::FailedToWriteFile)?;
    tokio::fs::create_dir_all(artifacts_path)
        .await
        .map_err(ApiError::FailedToWriteFile)?;

    // when the compilation is done, clean up the directories
    // it will be called when the AutoCleanUp struct is dropped
    let auto_clean_up = AutoCleanUp {
        dirs: vec![workspace_path.to_str().unwrap()],
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
    initialize_files(verification_request.contracts.clone(), workspace_path).await?;

    // Limit number of spawned processes. RAII released
    let _permit = SPAWN_SEMAPHORE.acquire().await.expect("Expired semaphore");

    let args = extract_verify_args(&verification_request);
    let command = tokio::process::Command::new("npx")
        .args(args)
        .current_dir(workspace_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let process = command.map_err(ApiError::FailedToExecuteCommand)?;
    let output = process
        .wait_with_output()
        .await
        .map_err(ApiError::FailedToReadOutput)?;
    let status = output.status;
    let message = String::from_utf8_lossy(&output.stdout).to_string();

    // calling here explicitly to avoid dropping the AutoCleanUp struct
    auto_clean_up.clean_up().await;

    if !status.success() {
        metrics
            .action_failures_total
            .with_label_values(&[VERIFICATION_LABEL_VALUE])
            .inc();

        return Ok(Json(VerifyResponse {
            status: "Error".to_string(),
            message: String::from_utf8_lossy(&output.stderr).to_string(),
        }));
    }

    metrics
        .action_successes_total
        .with_label_values(&[VERIFICATION_LABEL_VALUE])
        .inc();

    Ok(Json(VerifyResponse {
        status: "Success".to_string(),
        message,
    }))
}
