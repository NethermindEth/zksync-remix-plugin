use crate::errors::{ApiError, Result};
use crate::handlers::types::{CompilationConfig, CompilationRequest, CompiledFile};
use rocket::tokio;
use rocket::tokio::fs;
use solang_parser::diagnostics::{Diagnostic, ErrorType, Level};
use solang_parser::pt::Loc;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

pub const SOL_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "hardhat_env/workspaces/");
pub const ZK_CACHE_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    "hardhat_env/workspaces/cache-zk/"
);
pub const HARDHAT_ENV_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", "hardhat_env/");

pub const ARTIFACTS_ROOT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/", "hardhat_env/artifacts-zk");

pub const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub const DURATION_TO_PURGE: u64 = 60 * 5; // 5 minutes

pub const ZKSOLC_VERSIONS: [&str; 4] = ["1.5.6", "1.5.5", "1.4.1", "1.4.0"];

pub const DEFAULT_SOLIDITY_VERSION: &str = "0.8.24";

pub const DEFAULT_ZKSOLC_VERSION: &str = "1.5.6";

pub const ALLOWED_NETWORKS: [&str; 2] = ["sepolia", "mainnet"];

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
        ext if ext == "sol" => {
            let file_path = Path::new(SOL_ROOT).join(version).join(file_path);
            let file_name = file_path.file_name().unwrap().to_str().unwrap();

            // Trim .sol extension
            let file_name_without_ext = file_name.trim_end_matches(".sol");

            // make /<file_name_without_ext>/<file_name>.sol
            file_path
                .parent()
                .unwrap()
                .join(file_name_without_ext)
                .join(file_name)
        }
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

pub async fn clean_up(paths: Vec<String>) {
    for path in paths {
        let _ = fs::remove_dir_all(path).await;
    }

    let _ = fs::remove_dir_all(ZK_CACHE_ROOT).await;
}

pub fn generate_folder_name() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

pub fn list_files_in_directory<P: AsRef<Path>>(path: P) -> Vec<String> {
    let mut file_paths = Vec::new();

    for entry in WalkDir::new(path) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    file_paths.push(entry.path().display().to_string());
                }
            }
            Err(e) => println!("Error reading directory: {}", e),
        }
    }

    file_paths
}

pub fn generate_mock_compile_request() -> CompilationRequest {
    CompilationRequest {
        config: CompilationConfig {
            version: "1.4.1".to_string(),
            user_libraries: vec![],
        },
        contracts: vec![CompiledFile {
            file_name: "SimpleStorage.sol".to_string(),
            file_content: generate_mock_solidity_file_content(),
            is_contract: false,
        }],
        target_path: None,
    }
}

pub fn generate_mock_solidity_file_content() -> String {
    r#"
    pragma solidity ^0.8.0;

    contract SimpleStorage {
        uint256 storedData;

        function set(uint256 x) public {
            storedData = x;
        }

        function get() public view returns (uint256) {
            return storedData;
        }
    }
    "#
    .to_string()
}

pub async fn initialize_files(files: Vec<CompiledFile>, file_path: &Path) -> Result<()> {
    for file in files {
        let file_path_str = format!("{}/{}", file_path.to_str().unwrap(), file.file_name);
        let file_path = Path::new(&file_path_str);

        // create parent directories
        tokio::fs::create_dir_all(file_path.parent().unwrap())
            .await
            .map_err(ApiError::FailedToWriteFile)?;

        // write file
        tokio::fs::write(file_path, file.file_content.clone())
            .await
            .map_err(ApiError::FailedToWriteFile)?;
    }

    Ok(())
}
