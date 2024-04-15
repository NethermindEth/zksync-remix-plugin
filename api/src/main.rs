#[macro_use]
extern crate rocket;

pub mod cors;
pub mod handlers;
pub mod rate_limiter;
pub mod tracing_log;
pub mod types;
pub mod utils;
pub mod worker;

use crate::cors::CORS;
use crate::rate_limiter::RateLimiter;
use crate::tracing_log::init_logger;
use crate::utils::lib::{ARTIFACTS_ROOT, SOL_ROOT};
use crate::worker::WorkerEngine;
use clokwerk::{Scheduler, TimeUnits};
use handlers::compile::{compile, compile_async, get_compile_result};
use handlers::compiler_version::{allowed_versions, compiler_version};
use handlers::process::get_process_status;
use handlers::save_code::save_code;
use handlers::service_version::service_version;
use handlers::{health, who_is_this};
use handlers::verify::verify;
use rocket::tokio;
use rocket::tokio::time::sleep;
use tracing::info;

async fn clear_artifacts() {
    let _ = tokio::fs::remove_dir_all(ARTIFACTS_ROOT).await;
    let _ = tokio::fs::create_dir_all(ARTIFACTS_ROOT).await;
    let _ = tokio::fs::remove_dir_all(SOL_ROOT).await;
    let _ = tokio::fs::create_dir_all(SOL_ROOT).await;
    info!("artifacts cleared!");
}

#[launch]
async fn rocket() -> _ {
    if let Err(err) = init_logger() {
        eprintln!("Error initializing logger: {}", err);
    }

    let number_of_workers = match std::env::var("WORKER_THREADS") {
        Ok(v) => v.parse::<u32>().unwrap_or(2u32),
        Err(_) => 2u32,
    };

    let queue_size = match std::env::var("QUEUE_SIZE") {
        Ok(v) => v.parse::<usize>().unwrap_or(1_000),
        Err(_) => 1_000,
    };

    // Launch the worker processes
    let mut engine = WorkerEngine::new(number_of_workers, queue_size);

    engine.start();

    // Create a new scheduler
    let mut scheduler = Scheduler::new();

    scheduler.every(1.day()).run(move || {
        tokio::spawn(async {
            clear_artifacts().await;
        });
    });

    // Run the scheduler in a separate thread
    tokio::spawn(async move {
        loop {
            scheduler.run_pending();
            sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    info!("Number of workers: {}", number_of_workers);
    info!("Queue size: {}", queue_size);

    info!("Starting Rocket webserver...");

    rocket::build()
        .manage(engine)
        .manage(RateLimiter::new())
        .attach(CORS)
        .mount(
            "/",
            routes![
                compile,
                compile_async,
                get_compile_result,
                verify,
                save_code,
                compiler_version,
                get_process_status,
                allowed_versions,
                health,
                who_is_this,
                service_version
            ],
        )
}
