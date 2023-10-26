use crate::types::{ApiError, Result};
use rocket::tokio::fs;
use std::path::{Path, PathBuf};

pub const SOL_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "hardhat_env/contracts/");
pub const HARDHAT_ENV_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "hardhat_env/");
pub const ARTIFACTS_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    "hardhat_env/artifacts-zk/contracts/"
);

pub const DURATION_TO_PURGE: u64 = 60 * 5; // 5 minutes

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

pub fn get_file_path(file_path: &String) -> PathBuf {
    match get_file_ext(file_path).to_string() {
        // Leaving this here for potential use with vyper
        ext if ext == "sol" => Path::new(SOL_ROOT).join(file_path),
        _ => Path::new(SOL_ROOT).join(file_path),
    }
}

pub fn timestamp() -> u64 {
    chrono::Utc::now().timestamp() as u64
}
