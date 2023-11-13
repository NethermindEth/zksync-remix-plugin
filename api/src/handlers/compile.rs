use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult, CompileResponse, SolFile};
use crate::rate_limiter::RateLimited;
use crate::types::{ApiError, Result};
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    check_file_ext, get_file_path, path_buf_to_string, status_code_to_message,
    to_human_error_batch, ALLOWED_VERSIONS, ARTIFACTS_ROOT, SOL_ROOT,
};
use crate::worker::WorkerEngine;
use rocket::serde::json;
use rocket::serde::json::Json;
use rocket::tokio::fs;
use rocket::State;
use solang_parser::pt::SourceUnitPart;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::info;
use tracing::instrument;

#[instrument]
#[get("/compile/<version>/<remix_file_path..>")]
pub async fn compile(
    version: String,
    remix_file_path: PathBuf,
    _rate_limited: RateLimited,
) -> Json<CompileResponse> {
    info!("/compile/{:?}/{:?}", version, remix_file_path);
    do_compile(version, remix_file_path)
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
#[get("/compile-async/<version>/<remix_file_path..>")]
pub async fn compile_async(
    version: String,
    remix_file_path: PathBuf,
    _rate_limited: RateLimited,
    engine: &State<WorkerEngine>,
) -> String {
    info!("/compile-async/{:?}/{:?}", version, remix_file_path);
    do_process_command(
        ApiCommand::Compile {
            path: remix_file_path,
            version,
        },
        engine,
    )
}

#[instrument]
#[get("/compile-result/<process_id>")]
pub async fn get_compile_result(process_id: String, engine: &State<WorkerEngine>) -> String {
    info!("/compile-result/{:?}", process_id);
    fetch_process_result(process_id, engine, |result| match result {
        ApiCommandResult::Compile(compilation_result) => {
            json::to_string(&compilation_result).unwrap_or_default()
        }
        _ => String::from("Result not available"),
    })
}

pub async fn do_compile(
    version: String,
    remix_file_path: PathBuf,
) -> Result<Json<CompileResponse>> {
    if !ALLOWED_VERSIONS.contains(&version.as_str()) {
        return Err(ApiError::VersionNotSupported(version));
    }

    let remix_file_path = path_buf_to_string(remix_file_path.clone())?;

    check_file_ext(&remix_file_path, "sol")?;

    let file_path = get_file_path(&version, &remix_file_path);

    let sources_path = path_buf_to_string(Path::new(SOL_ROOT).join(&version))?;
    let artifacts_path = ARTIFACTS_ROOT.to_string();

    let hardhat_config = HardhatConfigBuilder::new()
        .zksolc_version(&version)
        .sources_path(&sources_path)
        .artifacts_path(&artifacts_path)
        .build();

    // save temporary hardhat config to file
    let hardhat_config_path = Path::new(SOL_ROOT).join(hardhat_config.name.clone());

    fs::write(
        hardhat_config_path.clone(),
        hardhat_config.to_string_config(),
    )
    .await
    .map_err(ApiError::FailedToWriteFile)?;

    let compile_result = Command::new("npx")
        .arg("hardhat")
        .arg("compile")
        .arg("--config")
        .arg(hardhat_config_path.clone())
        .current_dir(SOL_ROOT)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ApiError::FailedToExecuteCommand)?;

    let output = compile_result
        .wait_with_output()
        .map_err(ApiError::FailedToReadOutput)?;

    println!("output: {:?}", output);

    let result_path_prefix = Path::new(&artifacts_path)
        .join(&version)
        .join(remix_file_path.clone())
        .to_str()
        .ok_or(ApiError::FailedToParseString)?
        .to_string();

    println!("result_path_prefix: {}", result_path_prefix);
    println!("sources_path: {}", sources_path);
    println!("file_path: {}", path_buf_to_string(file_path.clone())?);

    let sol_file_content = fs::read_to_string(&file_path)
        .await
        .map_err(ApiError::FailedToReadFile)?;

    // delete the hardhat config file
    fs::remove_file(hardhat_config_path)
        .await
        .map_err(ApiError::FailedToWriteFile)?;

    let (ast, _) = solang_parser::parse(&sol_file_content, 0)
        .map_err(|e| ApiError::FailedToParseSol(to_human_error_batch(e)))?;

    // retrieve the contract names from the AST
    let mut compiled_contracts: Vec<SolFile> = Vec::new();
    for part in &ast.0 {
        if let SourceUnitPart::ContractDefinition(def) = part {
            if let Some(ident) = &def.name {
                let result_file_path = format!("{}/{}.json", result_path_prefix, ident);
                let file_content = fs::read_to_string(result_file_path)
                    .await
                    .map_err(ApiError::FailedToReadFile)?;
                let file_name = format!("{}.json", ident);

                compiled_contracts.push(SolFile {
                    file_name,
                    file_content,
                });
            }
        }
    }

    let status = status_code_to_message(output.status.code());
    let message = String::from_utf8(output.stderr)
        .map_err(ApiError::UTF8Error)?
        .replace(
            &file_path
                .to_str()
                .ok_or(ApiError::FailedToParseString)?
                .to_string(),
            &remix_file_path,
        )
        .replace(&result_path_prefix, &remix_file_path);

    if status != "Success" {
        return Ok(Json(CompileResponse {
            message,
            status,
            file_content: vec![],
        }));
    }

    Ok(Json(CompileResponse {
        message,
        status,
        file_content: compiled_contracts,
    }))
}
