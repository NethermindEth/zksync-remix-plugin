pub mod compile;
pub mod compiler_version;
pub mod process;
pub mod service_version;
pub mod types;
pub mod verify;

use crate::handlers::compile::do_compile;
use crate::handlers::compiler_version::do_compiler_version;
use crate::handlers::types::{ApiCommand, ApiCommandResult, HealthCheckResponse};
use crate::handlers::verify::do_verify;
use crate::types::ApiError;
use crate::utils::lib::generate_mock_compile_request;
use tracing::info;
use tracing::instrument;

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
        ApiCommand::Verify(request) => match do_verify(request).await {
            Ok(verify_response) => Ok(ApiCommandResult::Verify(verify_response.into_inner())),
            Err(e) => Err(e),
        },
        ApiCommand::Shutdown => Ok(ApiCommandResult::Shutdown),
    }
}
