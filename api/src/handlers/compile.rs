use crate::handlers::process::{do_process_command, fetch_process_result};
use crate::handlers::types::{ApiCommand, ApiCommandResult, CompileResponse, SolFile};
use crate::rate_limiter::RateLimited;
use crate::types::{ApiError, Result};
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    check_file_ext, clean_up, get_file_path, path_buf_to_string, status_code_to_message,
    to_human_error_batch, ZKSOLC_VERSIONS, ARTIFACTS_ROOT, CARGO_MANIFEST_DIR, SOL_ROOT,
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

async fn wrap_error(paths: Vec<String>, error: ApiError) -> ApiError {
    clean_up(paths).await;
    error
}

pub async fn do_compile(
    version: String,
    remix_file_path: PathBuf,
) -> Result<Json<CompileResponse>> {
    if !ZKSOLC_VERSIONS.contains(&version.as_str()) {
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

    let result_path_prefix = Path::new(&artifacts_path)
        .join(&version)
        .join(remix_file_path.clone());
    let result_path_filename = result_path_prefix
        .file_name()
        .ok_or(wrap_error(vec![], ApiError::FailedToParseString).await)?
        .to_str()
        .ok_or(wrap_error(vec![], ApiError::FailedToParseString).await)?;

    let result_path_filename_without_ext = result_path_filename.trim_end_matches(".sol");

    let result_path_prefix = result_path_prefix
        .parent()
        .ok_or(wrap_error(vec![], ApiError::FailedToGetParentDir).await)?
        .join(result_path_filename_without_ext)
        .join(result_path_filename)
        .to_str()
        .ok_or(wrap_error(vec![], ApiError::FailedToParseString).await)?
        .to_string();

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

    let compile_result = Command::new("npx")
        .arg("hardhat")
        .arg("compile")
        .arg("--config")
        .arg(hardhat_config_path.clone())
        .arg("--force")
        .current_dir(SOL_ROOT)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    if let Err(err) = compile_result {
        return Err(wrap_error(vec![file_path_dir], ApiError::FailedToExecuteCommand(err)).await);
    }

    // safe to unwrap because we checked for error above
    let compile_result = compile_result.unwrap();

    let output = compile_result.wait_with_output();
    if let Err(err) = output {
        return Err(wrap_error(
            vec![file_path_dir, result_path_prefix],
            ApiError::FailedToReadOutput(err),
        )
        .await);
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
        return Err(wrap_error(
            vec![file_path_dir, result_path_prefix],
            ApiError::FailedToExecuteCommand(err),
        )
        .await);
    }

    let clean_cache = clean_cache.unwrap();
    let _ = clean_cache.wait_with_output();

    // delete the hardhat config file
    let remove_file = fs::remove_file(hardhat_config_path).await;
    if let Err(err) = remove_file {
        return Err(wrap_error(
            vec![file_path_dir, result_path_prefix],
            ApiError::FailedToRemoveFile(err),
        )
        .await);
    }

    let message = match String::from_utf8(output.stderr) {
        Ok(msg) => msg,
        Err(err) => {
            return Err(wrap_error(
                vec![file_path_dir, result_path_prefix],
                ApiError::UTF8Error(err),
            )
            .await);
        }
    }
    .replace(&file_path, &remix_file_path)
    .replace(&result_path_prefix, &remix_file_path)
    .replace(CARGO_MANIFEST_DIR, "");

    let status = status_code_to_message(output.status.code());
    if status != "Success" {
        clean_up(vec![file_path_dir, result_path_prefix]).await;

        return Ok(Json(CompileResponse {
            message,
            status,
            file_content: vec![],
        }));
    }

    let sol_file_content = match fs::read_to_string(&file_path).await {
        Ok(content) => content,
        Err(err) => {
            return Err(wrap_error(
                vec![file_path_dir, result_path_prefix],
                ApiError::FailedToReadFile(err),
            )
            .await);
        }
    };

    let (ast, _) = match solang_parser::parse(&sol_file_content, 0) {
        Ok(result) => result,
        Err(err) => {
            return Err(wrap_error(
                vec![file_path_dir, result_path_prefix],
                ApiError::FailedToParseSol(to_human_error_batch(err)),
            )
            .await);
        }
    };

    // retrieve the contract names from the AST
    let mut compiled_contracts: Vec<SolFile> = Vec::new();
    for part in &ast.0 {
        if let SourceUnitPart::ContractDefinition(def) = part {
            if let Some(ident) = &def.name {
                let result_file_path = format!("{}/{}.json", result_path_prefix, ident);
                let file_content = match fs::read_to_string(result_file_path).await {
                    Ok(content) => content,
                    Err(err) => {
                        return Err(wrap_error(
                            vec![file_path_dir, result_path_prefix],
                            ApiError::FailedToReadFile(err),
                        )
                        .await);
                    }
                };
                let file_name = format!("{}.json", ident);

                compiled_contracts.push(SolFile {
                    file_name,
                    file_content,
                });
            }
        }
    }

    clean_up(vec![
        file_path_dir.to_string(),
        result_path_prefix.to_string(),
    ])
    .await;

    Ok(Json(CompileResponse {
        message,
        status,
        file_content: compiled_contracts,
    }))
}
