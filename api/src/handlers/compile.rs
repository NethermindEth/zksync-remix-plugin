use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult, CompileResponse};
use crate::utils::lib::{get_file_ext, get_file_path, SOL_ROOT};
use crate::worker::WorkerEngine;
use rocket::fs::NamedFile;
use rocket::serde::json;
use rocket::serde::json::Json;
use rocket::tokio::fs;
use rocket::State;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, instrument};

#[instrument]
#[get("/compile/<remix_file_path..>")]
pub async fn compile(remix_file_path: PathBuf) -> Json<CompileResponse> {
    info!("/compile");
    do_compile(remix_file_path)
        .await
        .unwrap_or(Json::from(CompileResponse {
            message: "Error compiling".to_string(),
            status: "error".to_string(),
            file_content: "".to_string(),
        }))
}

#[instrument]
#[get("/compile-async/<remix_file_path..>")]
pub async fn compile_async(
    remix_file_path: PathBuf,
    engine: &State<WorkerEngine>,
) -> String {
    info!("/compile-async");
    do_process_command(ApiCommand::Compile(remix_file_path), engine)
}

#[instrument]
#[get("/compile-result/<process_id>")]
pub async fn get_compile_result(process_id: String, engine: &State<WorkerEngine>) -> String {
    info!("/compile-result");
    fetch_process_result(process_id, engine, |result| match result {
        ApiCommandResult::Compile(compilation_result) => json::to_string(&compilation_result).unwrap(),
        _ => String::from("Result not available"),
    })
}

pub async fn do_compile(
    remix_file_path: PathBuf,
) -> Result<Json<CompileResponse>, String> {
    let remix_file_path = match remix_file_path.to_str() {
        Some(path) => path.to_string(),
        None => {
            return Ok(Json(CompileResponse {
                file_content: "".to_string(),
                message: "File path not found".to_string(),
                status: "FileNotFound".to_string(),
            }));
        }
    };

    match get_file_ext(&remix_file_path) {
        ext if ext == "sol" => {
            debug!("LOG: File extension is sol");
        }
        _ => {
            debug!("LOG: File extension not supported");
            return Ok(Json(CompileResponse {
                file_content: "".to_string(),
                message: "File extension not supported".to_string(),
                status: "FileExtensionNotSupported".to_string(),
            }));
        }
    }

    let file_path = get_file_path(&remix_file_path);

    let mut compile = Command::new("./zksolc");

    let sol_path = Path::new(SOL_ROOT).join(&remix_file_path);

    match sol_path.parent() {
        Some(parent) => match fs::create_dir_all(parent).await {
            Ok(_) => {
                debug!("LOG: Created directory: {:?}", parent);
            }
            Err(e) => {
                debug!("LOG: Error creating directory: {:?}", e);
            }
        },
        None => {
            debug!("LOG: Error creating directory");
        }
    }

    let result = compile
        .arg("--solc")
        .arg("./solc-linux-amd64-v0.8.19+commit.7dd6d404")
        .arg(&file_path)
        .arg("-o")
        .arg(&sol_path)
        .arg("-O")
        .arg("3")
        .arg("--overwrite")
        .arg("--combined-json")
        .arg("abi")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute compile: {:?}", e))?;

    debug!("LOG: ran command:{:?}", compile);

    let output = result.wait_with_output().expect("Failed to wait on child");

    Ok(Json(CompileResponse {
        file_content: match NamedFile::open(&sol_path).await.ok() {
            Some(file) => match file.path().to_str() {
                Some(path) => match fs::read_to_string(path.to_string()).await {
                    Ok(compiled) => compiled.to_string(),
                    Err(e) => e.to_string(),
                },
                None => "".to_string(),
            },
            None => "".to_string(),
        },
        message: String::from_utf8(output.stderr)
            .unwrap()
            .replace(&file_path.to_str().unwrap().to_string(), &remix_file_path)
            .replace(
                &sol_path.to_str().unwrap().to_string(),
                &remix_file_path,
            ),
        status: match output.status.code() {
            Some(0) => "Success".to_string(),
            Some(_) => "CompilationFailed".to_string(),
            None => "UnknownError".to_string(),
        },
    }))
}
