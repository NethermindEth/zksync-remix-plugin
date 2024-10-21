pub mod compile;
pub mod compiler_version;
pub mod process;
pub mod types;
pub mod utils;
pub mod verify;

use lazy_static::lazy_static;
use rocket::State;
use tokio::sync::Semaphore;
use tokio::time::Instant;
use tracing::info;
use tracing::instrument;

use crate::errors::ApiError;
use crate::handlers::compile::{do_compile, COMPILATION_LABEL_VALUE};
use crate::handlers::compiler_version::do_compiler_version;
use crate::handlers::types::{ApiCommand, ApiCommandResult, HealthCheckResponse};
use crate::handlers::verify::{do_verify, VERIFICATION_LABEL_VALUE};
use crate::metrics::Metrics;
use crate::utils::lib::generate_mock_compile_request;
use crate::worker::WorkerEngine;

const PROCESS_SPAWN_LIMIT: usize = 8;
lazy_static! {
    static ref SPAWN_SEMAPHORE: Semaphore = Semaphore::new(PROCESS_SPAWN_LIMIT);
}

#[instrument(skip(engine))]
#[get("/health")]
pub async fn health(engine: &State<WorkerEngine>) -> HealthCheckResponse {
    info!("/health");

    let result = do_compile(generate_mock_compile_request(), &engine.metrics, true).await;

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

pub async fn dispatch_command(
    command: ApiCommand,
    metrics: &Metrics,
) -> Result<ApiCommandResult, ApiError> {
    let start_time = Instant::now();

    match command {
        ApiCommand::CompilerVersion => match do_compiler_version() {
            Ok(result) => Ok(ApiCommandResult::CompilerVersion(result)),
            Err(e) => Err(e),
        },
        ApiCommand::Compile(request) => {
            let res = match do_compile(request, metrics, false).await {
                Ok(compile_response) => {
                    Ok(ApiCommandResult::Compile(compile_response.into_inner()))
                }
                Err(e) => {
                    metrics
                        .action_failures_total
                        .with_label_values(&[COMPILATION_LABEL_VALUE])
                        .inc();
                    Err(e)
                }
            };

            let elapsed_time = start_time.elapsed().as_secs_f64();
            metrics
                .action_duration_seconds
                .with_label_values(&[COMPILATION_LABEL_VALUE])
                .set(elapsed_time);

            res
        }
        ApiCommand::Verify(request) => {
            let res = match do_verify(request, metrics).await {
                Ok(verify_response) => Ok(ApiCommandResult::Verify(verify_response.into_inner())),
                Err(e) => {
                    metrics
                        .action_failures_total
                        .with_label_values(&[VERIFICATION_LABEL_VALUE])
                        .inc();

                    Err(e)
                }
            };

            let elapsed_time = start_time.elapsed().as_secs_f64();
            metrics
                .action_duration_seconds
                .with_label_values(&[VERIFICATION_LABEL_VALUE])
                .set(elapsed_time);

            res
        }
        ApiCommand::Shutdown => Ok(ApiCommandResult::Shutdown),
    }
}
