use std::path::{Path, PathBuf};
use tracing::debug;
use types::ARTIFACTS_FOLDER;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::commands::compile::CompilationFile;

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

pub const ZKSOLC_VERSIONS: [&str; 2] = ["1.4.1", "1.4.0"];

pub const DEFAULT_SOLIDITY_VERSION: &str = "0.8.24";

pub const DEFAULT_ZKSOLC_VERSION: &str = "1.4.1";

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

// pub fn to_human_error(
//     Diagnostic {
//         loc,
//         level,
//         ty,
//         message,
//         notes,
//     }: Diagnostic,
// ) -> String {
//     let level = match level {
//         Level::Debug => "Debug",
//         Level::Info => "Info",
//         Level::Warning => "Warning",
//         Level::Error => "Error",
//     };
//
//     let loc = match loc {
//         Loc::Builtin => "Builtin".to_string(),
//         Loc::CommandLine => "CommandLine".to_string(),
//         Loc::Implicit => "Implicit".to_string(),
//         Loc::Codegen => "Codegen".to_string(),
//         Loc::File(_, start, end) => format!("{}:{}", start, end),
//     };
//
//     let ty = match ty {
//         ErrorType::None => "None",
//         ErrorType::ParserError => "ParserError",
//         ErrorType::SyntaxError => "SyntaxError",
//         ErrorType::DeclarationError => "DeclarationError",
//         ErrorType::CastError => "CastError",
//         ErrorType::TypeError => "TypeError",
//         ErrorType::Warning => "Warning",
//     };
//
//     let notes = notes
//         .iter()
//         .map(|note| note.message.clone())
//         .collect::<Vec<String>>()
//         .join("\n");
//
//     format!(
//         "level: {}, loc: {}, ty: {}, message: {}, notes: {}\n",
//         level, loc, ty, message, notes
//     )
// }

// pub fn to_human_error_batch(diagnostics: Vec<Diagnostic>) -> String {
//     diagnostics
//         .into_iter()
//         .map(to_human_error)
//         .collect::<Vec<String>>()
//         .join("\n")
// }

// pub async fn clean_up(paths: Vec<String>) {
//     for path in paths {
//         let _ = fs::remove_dir_all(path).await;
//     }
//
//     let _ = fs::remove_dir_all(ZK_CACHE_ROOT).await;
// }

pub fn generate_folder_name() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

pub fn list_files_in_directory<P: AsRef<Path>>(path: P) -> Result<Vec<String>, walkdir::Error> {
    let mut file_paths = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            file_paths.push(entry.path().display().to_string());
        }
    }

    Ok(file_paths)
}

// pub fn generate_mock_compile_request() -> CompilationRequest {
//     CompilationRequest {
//         config: CompilationConfig {
//             version: "1.4.1".to_string(),
//             user_libraries: vec![],
//         },
//         contracts: vec![CompiledFile {
//             file_name: "SimpleStorage.sol".to_string(),
//             file_content: generate_mock_solidity_file_content(),
//             is_contract: false,
//         }],
//         target_path: None,
//     }
// }

pub fn s3_artifacts_dir(id: &str) -> String {
    format!("{}/{}/", ARTIFACTS_FOLDER, id)
}

pub fn s3_compilation_files_dir(id: &str) -> String {
    format!("{}/", id)
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

pub async fn initialize_files(
    dst_dir: impl AsRef<Path>,
    files: Vec<CompilationFile>,
) -> Result<(), std::io::Error> {
    for file in files {
        let file_path = dst_dir.as_ref().join(file.file_path);

        // create parent directories
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;

        // write file
        tokio::fs::write(file_path, file.file_content.clone()).await?;
    }

    Ok(())
}
