// TODO: extract in class

use anyhow::Context;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use tracing::{error, info};
use types::CompilationConfig;

use crate::commands::errors::CompilationError;
use crate::commands::{CompilationFile, SPAWN_SEMAPHORE};
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{initialize_files, list_files_in_directory, DEFAULT_SOLIDITY_VERSION};

pub struct CompilationInput {
    pub workspace_path: PathBuf,
    pub config: CompilationConfig,
    // legacy. files really
    pub contracts: Vec<CompilationFile>,
}

pub struct ArtifactData {
    pub file_path: PathBuf,
    pub is_contract: bool,
}

pub struct CompilationOutput {
    pub artifacts_dir: PathBuf,
    pub artifacts_data: Vec<ArtifactData>,
}

fn process_output(process_output: Output) -> Result<(), CompilationError> {
    const NOTHING_TO_COMPILE: &str = "Nothing to compile";

    if !process_output.status.success() {
        let err_msg = String::from_utf8_lossy(&process_output.stderr);
        error!("Compilation error: {}", err_msg);
        Err(CompilationError::CompilationFailureError(
            err_msg.to_string(),
        ))
    } else {
        let message = String::from_utf8_lossy(&process_output.stdout).to_string();
        if message.contains(NOTHING_TO_COMPILE) {
            Err(CompilationError::NothingToCompileError)
        } else {
            Ok(())
        }
    }
}

pub async fn do_compile(
    compilation_input: CompilationInput,
) -> Result<CompilationOutput, CompilationError> {
    // root directory for the contracts
    let workspace_path = compilation_input.workspace_path;
    // root directory for the artifacts
    let artifacts_path = workspace_path.join("artifacts-zk");
    // root directory for user files (hardhat config, etc)
    let hardhat_config_path = workspace_path.join("hardhat.config.ts");

    // instantly create the directories
    tokio::fs::create_dir_all(&workspace_path)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| {
            format!(
                "Couldn't create workspace dir: {}",
                workspace_path.display()
            )
        })?;
    tokio::fs::create_dir_all(&artifacts_path)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| {
            format!(
                "Couldn't create artifacts dir: {}",
                artifacts_path.display()
            )
        })?;

    // write the hardhat config file
    let zksolc_version = compilation_input.config.version;
    let mut hardhat_config_builder = HardhatConfigBuilder::new();
    hardhat_config_builder
        .zksolc_version(&zksolc_version)
        .solidity_version(DEFAULT_SOLIDITY_VERSION);
    if let Some(target_path) = compilation_input.config.target_path {
        hardhat_config_builder.paths_sources(&target_path);
    }

    let hardhat_config_content = hardhat_config_builder.build().to_string_config();

    // create parent directories
    tokio::fs::create_dir_all(hardhat_config_path.parent().unwrap())
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| {
            format!(
                "Couldn't create hardhat dir: {}",
                hardhat_config_path.display()
            )
        })?;
    tokio::fs::write(hardhat_config_path, hardhat_config_content)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Couldn't write hardhat.config file")?;

    // filter test files from compilation candidates
    let contracts = compilation_input
        .contracts
        .into_iter()
        .filter(|contract| !contract.file_path.ends_with("_test.sol"))
        .collect();

    // initialize the files
    initialize_files(&workspace_path, contracts)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Couldn't write contract to fs")?;

    // Limit number of spawned processes. RAII released
    let _permit = SPAWN_SEMAPHORE.acquire().await.expect("Expired semaphore");

    let process = tokio::process::Command::new("npx")
        .arg("hardhat")
        .arg("compile")
        .current_dir(&workspace_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(anyhow::Error::from)
        .with_context(|| "Couldn't spawn process")?;

    let output = process
        .wait_with_output()
        .await
        .map_err(anyhow::Error::from)?;
    process_output(output)?;

    // fetch the files in the artifacts directory
    let file_paths = list_files_in_directory(&artifacts_path)
        .map_err(anyhow::Error::from)
        .with_context(|| "Unexpected error listing artifact")?;

    let artifacts_data = file_paths
        .into_iter()
        .map(|file_path| {
            let full_path = Path::new(&file_path);
            let relative_path = full_path
                .strip_prefix(&artifacts_path)
                .expect("Unexpected prefix");

            let is_contract =
                !relative_path.ends_with(".dbg.json") && relative_path.ends_with(".json");

            ArtifactData {
                file_path: relative_path.to_path_buf(),
                is_contract,
            }
        })
        .collect();

    Ok(CompilationOutput {
        artifacts_dir: artifacts_path,
        artifacts_data,
    })
}
