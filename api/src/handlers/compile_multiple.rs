use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult, CompileResponse, SolFile};
use crate::rate_limiter::RateLimited;
use crate::types::{ApiError, Result};
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    ZKSOLC_VERSIONS, ARTIFACTS_ROOT, SOL_ROOT,
};
use crate::worker::WorkerEngine;
use rocket::serde::json::Json;
use rocket::{State, tokio};
use std::path::{Path};
use std::process::{Command};
use tracing::info;
use tracing::instrument;
use uuid::Uuid;


#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct MultifileCompilationRequest {
    pub contracts: Vec<SolFile>
}

#[instrument]
#[post("/compile-multiple/<version>", format = "json", data = "<request_json>")]
pub async fn compile_multiple(
    version: String,
    request_json: Json<MultifileCompilationRequest>,
    _rate_limited: RateLimited,
) -> Json<CompileResponse> {
    info!("/compile-multiple/{:?}", version);
    do_compile(version, request_json.0)
        .await
        .unwrap_or_else(|e| {
            Json(CompileResponse {
                file_content: vec![],
                message: e.to_string(),
                status: "Error".to_string(),
            })
        })
}

#[instrument]
#[get("/compile-multiple-async/<version>")]
pub async fn compile_multiple_async(
    version: String,
    _rate_limited: RateLimited,
    engine: &State<WorkerEngine>,
) -> String {
    info!("/compile-multiple-async/{:?}", version);

    unimplemented!()
}

#[instrument]
#[get("/compile-multiple-result/<process_id>")]
pub async fn get_compile_multiple_result(process_id: String, engine: &State<WorkerEngine>) -> String {
    info!("/compile-multiple-result/{:?}", process_id);

    unimplemented!()
}

pub struct AutoCleanUp<'a> {
    dirs: Vec<&'a str>,
}

impl Drop for AutoCleanUp<'_> {
    fn drop(&mut self) {
        self.clean_up_sync();
    }
}

impl AutoCleanUp<'_> {
    pub async fn clean_up(&self) {
        for path in self.dirs.iter() {
            println!("Removing path: {:?}", path);

            // check if the path exists
            if !Path::new(path).exists() {
                continue;
            }

            if let Err(e) = tokio::fs::remove_dir_all(path).await {
                info!("Failed to remove file: {:?}", e);
            }
        }
    }

    pub fn clean_up_sync(&self) {
        for path in self.dirs.iter() {
            println!("Removing path: {:?}", path);

            // check if the path exists
            if !Path::new(path).exists() {
                continue;
            }

            if let Err(e) = std::fs::remove_dir_all(path) {
                info!("Failed to remove file: {:?}", e);
            }
        }
    }

}

pub fn generate_folder_name() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

pub async fn do_compile(
    version: String,
    compilation_request: MultifileCompilationRequest
) -> Result<Json<CompileResponse>> {
    if !ZKSOLC_VERSIONS.contains(&version.as_str()) {
        return Err(ApiError::VersionNotSupported(version));
    }

    let namespace = generate_folder_name();

    let contracts_path_str = format!("{}/{}", SOL_ROOT, namespace);
    let contracts_path = Path::new(&contracts_path_str);

    let artifacts_path_str = format!("{}/{}", ARTIFACTS_ROOT, namespace);
    let artifacts_path = Path::new(&artifacts_path_str);

    // instantly create the directories
    tokio::fs::create_dir_all(contracts_path).await.map_err(ApiError::FailedToWriteFile)?;
    tokio::fs::create_dir_all(artifacts_path).await.map_err(ApiError::FailedToWriteFile)?;

    // when the compilation is done, clean up the directories
    // it will be called when the AutoCleanUp struct is dropped
    let auto_clean_up = AutoCleanUp {
        dirs: vec![contracts_path.to_str().unwrap(), artifacts_path.to_str().unwrap()],
    };

    for contract in compilation_request.contracts.iter() {
        let file_path_str = format!("{}/{}", contracts_path.to_str().unwrap(), contract.file_name);
        let file_path = Path::new(&file_path_str);

        // create parent directories
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await.map_err(ApiError::FailedToWriteFile)?;

        // write file
        tokio::fs::write(file_path, contract.file_content.clone()).await.map_err(ApiError::FailedToWriteFile)?;
    }

    let command = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/app/contracts", contracts_path.to_str().unwrap()))
        .arg("-v")
        .arg(format!("{}:/app/artifacts-zk", artifacts_path.to_str().unwrap()))
        .arg(format!("hardhat-env:{}", version))
        .arg("npx")
        .arg("hardhat")
        .arg("compile")
        .spawn();

    let process = command.map_err(ApiError::FailedToExecuteCommand)?;

    let _output = process.wait_with_output().map_err(ApiError::FailedToReadOutput)?;

    // calling here explicitly to avoid dropping the AutoCleanUp struct
    auto_clean_up.clean_up().await;

    Ok(Json(CompileResponse {
        file_content: vec![],
        message: "Success".to_string(),
        status: "Success".to_string(),
    }))
}
