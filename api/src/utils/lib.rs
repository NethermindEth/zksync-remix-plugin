use crate::types::{ApiError, Result};
use rocket::tokio::fs;
use solang_parser::diagnostics::{Diagnostic, ErrorType, Level};
use solang_parser::pt::Loc;
use std::path::{Path, PathBuf};

pub const SOL_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "hardhat_env/contracts/");
pub const HARDHAT_ENV_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "hardhat_env/");
pub const ARTIFACTS_ROOT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/", "hardhat_env/artifacts-zk");

pub const DURATION_TO_PURGE: u64 = 60 * 5; // 5 minutes

pub const ALLOWED_VERSIONS: [&str; 2] = ["latest", "1.3.13"];

#[allow(dead_code)]
pub const TEMP_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "temp/");

pub fn get_file_ext(file_path: &str) -> String {
    match file_path.split('.').last() {
        Some(ext) => ext.to_string(),
        None => {
            debug!("LOG: File extension not found");
            "".to_string()
        }
    }
}

pub fn check_file_ext(file_path: &str, ext: &str) -> Result<()> {
    let actual_ext = get_file_ext(file_path);
    if actual_ext == *ext {
        Ok(())
    } else {
        Err(ApiError::FileExtensionNotSupported(actual_ext))
    }
}

pub fn path_buf_to_string(path_buf: PathBuf) -> Result<String> {
    path_buf
        .to_str()
        .ok_or(ApiError::FailedToParseString)
        .map(|s| s.to_string())
}

pub async fn init_parent_directories(file_path: PathBuf) {
    match file_path.parent() {
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
}

pub fn status_code_to_message(status: Option<i32>) -> String {
    match status {
        Some(0) => "Success",
        Some(_) => "CompilationFailed",
        None => "UnknownError",
    }
    .to_string()
}

pub fn get_file_path(version: &str, file_path: &str) -> PathBuf {
    match get_file_ext(file_path).to_string() {
        // Leaving this here for potential use with vyper
        ext if ext == "sol" => Path::new(SOL_ROOT).join(version).join(file_path),
        _ => Path::new(SOL_ROOT).join(version).join(file_path),
    }
}

pub fn timestamp() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

pub fn to_human_error(
    Diagnostic {
        loc,
        level,
        ty,
        message,
        notes,
    }: Diagnostic,
) -> String {
    let level = match level {
        Level::Debug => "Debug",
        Level::Info => "Info",
        Level::Warning => "Warning",
        Level::Error => "Error",
    };

    let loc = match loc {
        Loc::Builtin => "Builtin".to_string(),
        Loc::CommandLine => "CommandLine".to_string(),
        Loc::Implicit => "Implicit".to_string(),
        Loc::Codegen => "Codegen".to_string(),
        Loc::File(_, start, end) => format!("{}:{}", start, end),
    };

    let ty = match ty {
        ErrorType::None => "None",
        ErrorType::ParserError => "ParserError",
        ErrorType::SyntaxError => "SyntaxError",
        ErrorType::DeclarationError => "DeclarationError",
        ErrorType::CastError => "CastError",
        ErrorType::TypeError => "TypeError",
        ErrorType::Warning => "Warning",
    };

    let notes = notes
        .iter()
        .map(|note| note.message.clone())
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "level: {}, loc: {}, ty: {}, message: {}, notes: {}\n",
        level, loc, ty, message, notes
    )
}

pub fn to_human_error_batch(diagnostics: Vec<Diagnostic>) -> String {
    diagnostics
        .into_iter()
        .map(to_human_error)
        .collect::<Vec<String>>()
        .join("\n")
}
