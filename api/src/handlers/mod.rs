pub mod compiler_version;
pub mod compile;
pub mod process;
pub mod save_code;
pub mod types;

use crate::handlers::compiler_version::do_compiler_version;
use crate::handlers::compile::do_compile;
use crate::handlers::types::{ApiCommand, ApiCommandResult, FileContentMap};
use tracing::info;
use std::path::Path;
use tracing::instrument;

#[instrument]
#[get("/health")]
pub async fn health() -> &'static str {
    info!("/health");
    "OK"
}

#[instrument]
#[get("/")]
pub async fn who_is_this() -> &'static str {
    info!("/who_is_this");
    "Who are you?"
}

pub async fn dispatch_command(command: ApiCommand) -> Result<ApiCommandResult, String> {
    match command {
        ApiCommand::CompilerVersion => match do_compiler_version() {
            Ok(result) => Ok(ApiCommandResult::CompilerVersion(result)),
            Err(e) => Err(e),
        },
        ApiCommand::Compile(remix_file_path) => {
            match do_compile(remix_file_path).await {
                Ok(compile_response) => Ok(ApiCommandResult::Compile(
                    compile_response.into_inner(),
                )),
                Err(e) => Err(e),
            }
        }
        ApiCommand::Shutdown => Ok(ApiCommandResult::Shutdown),
    }
}

fn get_files_recursive(base_path: &Path) -> Vec<FileContentMap> {
    let mut file_content_map_array: Vec<FileContentMap> = Vec::new();

    if base_path.is_dir() {
        for entry in base_path.read_dir().unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                file_content_map_array.extend(get_files_recursive(&path));
            } else if let Ok(content) = std::fs::read_to_string(&path) {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let file_content = content;
                let file_content_map = FileContentMap {
                    file_name,
                    file_content,
                };
                file_content_map_array.push(file_content_map);
            }
        }
    }

    file_content_map_array
}
