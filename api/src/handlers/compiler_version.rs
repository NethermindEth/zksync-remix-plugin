use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult};
use crate::worker::WorkerEngine;
use rocket::State;
use std::process::{Command, Stdio};
use tracing::{error, info, instrument};

#[instrument]
#[get("/compiler_version")]
pub async fn compiler_version() -> String {
    info!("/compiler_version");
    do_compiler_version().unwrap_or_else(|e| e)
}

#[instrument]
#[get("/compiler_version_async")]
pub async fn compiler_version_async(engine: &State<WorkerEngine>) -> String {
    info!("/compiler_version_async");
    do_process_command(ApiCommand::CompilerVersion, engine)
}

#[instrument]
#[get("/compiler_version_result/<process_id>")]
pub async fn get_compiler_version_result(process_id: String, engine: &State<WorkerEngine>) -> String {
    info!("/compiler_version_result/{:?}", process_id);
    fetch_process_result(process_id, engine, |result| match result {
        ApiCommandResult::CompilerVersion(version) => version.to_string(),
        _ => String::from("Result not available"),
    })
}

/// Run ./zksolc --version to return compiler version string
///
pub fn do_compiler_version() -> Result<String, String> {
    let mut version_caller = Command::new("./zksolc");
    match String::from_utf8(
        version_caller
            .arg("--version")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to execute zksolc")
            .wait_with_output()
            .expect("Failed to wait on child")
            .stdout,
    ) {
        Ok(version) => Ok(version),
        Err(e) => {
            error!("{:?}", e.to_string());
            Err(e.to_string())
        }
    }
}
