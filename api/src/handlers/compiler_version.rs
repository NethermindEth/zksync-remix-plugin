use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult};
use crate::types::ApiError;
use crate::worker::WorkerEngine;
use rocket::State;
use tracing::{info, instrument};

#[instrument]
#[get("/compiler_version")]
pub async fn compiler_version() -> String {
    info!("/compiler_version");
    do_compiler_version().unwrap_or_else(|e| e.to_string())
}

#[instrument]
#[get("/compiler_version_async")]
pub async fn compiler_version_async(engine: &State<WorkerEngine>) -> String {
    info!("/compiler_version_async");
    do_process_command(ApiCommand::CompilerVersion, engine)
}

#[instrument]
#[get("/compiler_version_result/<process_id>")]
pub async fn get_compiler_version_result(
    process_id: String,
    engine: &State<WorkerEngine>,
) -> String {
    info!("/compiler_version_result/{:?}", process_id);
    fetch_process_result(process_id, engine, |result| match result {
        ApiCommandResult::CompilerVersion(version) => version.to_string(),
        _ => String::from("Result not available"),
    })
}

/// Run ./zksolc --version to return compiler version string
///
pub fn do_compiler_version() -> Result<String, ApiError> {
    Ok("zksolc-latest".to_string())
}
