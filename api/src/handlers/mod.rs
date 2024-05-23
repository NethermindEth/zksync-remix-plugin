pub mod compile;
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
use crate::utils::lib::{get_file_path, init_parent_directories, ARTIFACTS_ROOT, generate_mock_compile_request};
use rocket::tokio;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;
use tracing::instrument;
use uuid::Uuid;
use crate::handlers::compile::do_compile;

#[instrument]
#[get("/health")]
pub async fn health() -> HealthCheckResponse {
    info!("/health");

    let result = do_compile(generate_mock_compile_request()).await;

    if result.is_ok() {
        HealthCheckResponse::ok()
    } else {
        HealthCheckResponse::error("Failed to compile")
    }
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
