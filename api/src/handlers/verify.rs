use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult, VerifyResponse};
use crate::rate_limiter::RateLimited;
use crate::types::{ApiError, Result};
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    check_file_ext, clean_up, get_file_path, path_buf_to_string, status_code_to_message,
    ALLOWED_VERSIONS, ARTIFACTS_ROOT, CARGO_MANIFEST_DIR, SOL_ROOT,
};
use crate::worker::WorkerEngine;
use rocket::serde::{Deserialize, json::Json, json};
use rocket::tokio::fs;
use rocket::State;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::info;
use tracing::instrument;

const ALLOWED_NETWORKS: [&str; 2] = ["sepolia", "mainnet"];

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct VerifyData {
    inputs: Vec<String>
}

#[instrument]
#[post("/verify/<version>/<network>/<contract_address>/<remix_file_path..>", data = "<data>")]
pub async fn verify(
    version: String,
    network: String,
    contract_address: String,
    remix_file_path: PathBuf,
    _rate_limited: RateLimited,
    data: Json<VerifyData>
) -> Json<VerifyResponse> {
    info!(
        "/verify/{:?}/{:?}/{:?}/{:?} data: {:?}",
        version, network, contract_address, remix_file_path,
        data.inputs,
    );
    do_verify(version, network, contract_address, remix_file_path, data.inputs.to_vec())
        .await
        .unwrap_or_else(|e| {
            Json(VerifyResponse {
                message: e.to_string(),
                status: "Error".to_string(),
            })
        })
}

#[instrument]
#[post("/verify-async/<version>/<network>/<contract_address>/<remix_file_path..>", data = "<data>")]
pub fn verify_async(
    version: String,
    network: String,
    contract_address: String,
    remix_file_path: PathBuf,
    _rate_limited: RateLimited,
    engine: &State<WorkerEngine>,
    data: Json<VerifyData>
) -> String {
    info!(
        "/verify-async/{:?}/{:?}/{:?}/{:?} data: {:?}",
        version, network, contract_address, remix_file_path, data.inputs
    );
    do_process_command(
        ApiCommand::Verify {
            path: remix_file_path,
            contract_address,
            network,
            version,
            inputs: data.inputs.to_vec(),
        },
        engine,
    )
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

async fn wrap_error(paths: Vec<String>, error: ApiError) -> ApiError {
    clean_up(paths).await;
    error
}

pub async fn do_verify(
    version: String,
    network: String,
    contract_address: String,
    remix_file_path: PathBuf,
    inputs: Vec<String>,
) -> Result<Json<VerifyResponse>> {
    if !ALLOWED_VERSIONS.contains(&version.as_str()) {
        return Err(wrap_error(vec![], ApiError::VersionNotSupported(version)).await);
    }

    let remix_file_path = path_buf_to_string(remix_file_path.clone())?;

    check_file_ext(&remix_file_path, "sol")?;

    let file_path = get_file_path(&version, &remix_file_path)
        .to_str()
        .ok_or(wrap_error(vec![], ApiError::FailedToParseString).await)?
        .to_string();

    let file_path_dir = Path::new(&file_path)
        .parent()
        .ok_or(wrap_error(vec![], ApiError::FailedToGetParentDir).await)?
        .to_str()
        .ok_or(ApiError::FailedToParseString)?
        .to_string();

    println!("file_path: {:?}", file_path);

    let artifacts_path = ARTIFACTS_ROOT.to_string();

    let hardhat_config = HardhatConfigBuilder::new()
        .zksolc_version(&version)
        .sources_path(&file_path_dir)
        .artifacts_path(&artifacts_path)
        .build();

    // save temporary hardhat config to file
    let hardhat_config_path = Path::new(SOL_ROOT).join(hardhat_config.name.clone());

    let result = fs::write(
        hardhat_config_path.clone(),
        hardhat_config.to_string_config(),
    )
    .await;

    if let Err(err) = result {
        return Err(wrap_error(vec![file_path_dir], ApiError::FailedToWriteFile(err)).await);
    }

    if !ALLOWED_NETWORKS.contains(&network.as_str()) {
        return Err(wrap_error(vec![file_path_dir], ApiError::UnknownNetwork(network)).await);
    }
    let network_arg = if network == "sepolia" {
        "zkSyncTestnet"
    } else {
        "zkSyncMainnet"
    };

    let verify_result = Command::new("npx")
        .arg("hardhat")
        .arg("verify")
        .arg("--config")
        .arg(hardhat_config_path.clone())
        .args(["--network", network_arg])
        .arg(contract_address)
        .args(inputs)
        .current_dir(SOL_ROOT)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    if let Err(err) = verify_result {
        return Err(wrap_error(vec![file_path_dir], ApiError::FailedToExecuteCommand(err)).await);
    }

    // safe to unwrap because we checked for error above
    let verify_result = verify_result.unwrap();

    let output = verify_result.wait_with_output();
    if let Err(err) = output {
        return Err(wrap_error(vec![file_path_dir], ApiError::FailedToReadOutput(err)).await);
    }
    let output = output.unwrap();

    let clean_cache = Command::new("npx")
        .arg("hardhat")
        .arg("clean")
        .current_dir(SOL_ROOT)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    if let Err(err) = clean_cache {
        return Err(wrap_error(vec![file_path_dir], ApiError::FailedToExecuteCommand(err)).await);
    }

    let clean_cache = clean_cache.unwrap();
    let _ = clean_cache.wait_with_output();

    // delete the hardhat config file
    let remove_file = fs::remove_file(hardhat_config_path).await;
    if let Err(err) = remove_file {
        return Err(wrap_error(vec![file_path_dir], ApiError::FailedToRemoveFile(err)).await);
    }

    let message = match String::from_utf8(output.stderr) {
        Ok(msg) => msg,
        Err(err) => {
            return Err(wrap_error(vec![file_path_dir], ApiError::UTF8Error(err)).await);
        }
    }
    .replace(&file_path, &remix_file_path)
    .replace(CARGO_MANIFEST_DIR, "");

    let status = status_code_to_message(output.status.code());
    if status != "Success" {
        clean_up(vec![file_path_dir]).await;

        return Ok(Json(VerifyResponse { message, status }));
    }

    clean_up(vec![file_path_dir.to_string()]).await;

    Ok(Json(VerifyResponse { message, status }))
}
