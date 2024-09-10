// TODO: extract in class

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tracing::{error, info};
use types::CompilationConfig;

use crate::commands::errors::CompilationError;
use crate::commands::SPAWN_SEMAPHORE;
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{initialize_files, list_files_in_directory, DEFAULT_SOLIDITY_VERSION};

pub struct CompilationFile {
    pub file_path: String,
    pub file_content: Vec<u8>,
}

pub struct CompilationInput {
    pub workspace_path: PathBuf,
    pub config: CompilationConfig,
    // legacy. files really
    pub contracts: Vec<CompilationFile>,
}

pub struct CompilationArtifact {
    pub file_name: String,
    pub file_content: Vec<u8>,
    pub is_contract: bool,
}
pub async fn do_compile(
    compilation_input: CompilationInput,
) -> Result<Vec<CompilationArtifact>, CompilationError> {
    // root directory for the contracts
    let workspace_path = compilation_input.workspace_path;
    // root directory for the artifacts
    let artifacts_path = workspace_path.join("artifacts-zk");
    // root directory for user files (hardhat config, etc)
    let hardhat_config_path = workspace_path.join("hardhat.config.ts");

    // instantly create the directories
    tokio::fs::create_dir_all(&workspace_path).await?;
    tokio::fs::create_dir_all(&artifacts_path).await?;

    // when the compilation is done, clean up the directories
    // it will be called when the AutoCleanUp struct is dropped
    let auto_clean_up = AutoCleanUp {
        dirs: vec![workspace_path.to_str().unwrap()],
    };

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
    tokio::fs::create_dir_all(hardhat_config_path.parent().unwrap()).await?;
    tokio::fs::write(hardhat_config_path, hardhat_config_content).await?;

    // filter test files from compilation candidates
    let contracts = compilation_input
        .contracts
        .into_iter()
        .filter(|contract| !contract.file_path.ends_with("_test.sol"))
        .collect();

    // initialize the files
    initialize_files(&workspace_path, contracts).await?;

    // Limit number of spawned processes. RAII released
    let _permit = SPAWN_SEMAPHORE.acquire().await.expect("Expired semaphore");

    let process = tokio::process::Command::new("npx")
        .arg("hardhat")
        .arg("compile")
        .current_dir(&workspace_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = process.wait_with_output().await?;

    let status = output.status;
    let message = String::from_utf8_lossy(&output.stdout).to_string();
    info!("Output: \n{:?}", message);

    if !status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        error!("Compilation error: {}", err_msg);
        return Err(CompilationError::CompilationFailureError(
            err_msg.to_string(),
        ));
    }

    // fetch the files in the artifacts directory
    let mut file_contents: Vec<CompilationArtifact> = vec![];
    let file_paths =
        list_files_in_directory(&artifacts_path).expect("Unexpected error listing artifact");
    for file_path in file_paths.iter() {
        // TODO: change this - don't store files in RAM. copy 1-1 to S3
        let file_content = tokio::fs::read(file_path).await?;
        let full_path = Path::new(file_path);

        let relative_path = full_path
            .strip_prefix(&artifacts_path)
            .expect("Unexpected prefix");
        let relative_path_str = relative_path.to_str().unwrap();

        let is_contract =
            !relative_path_str.ends_with(".dbg.json") && relative_path_str.ends_with(".json");

        file_contents.push(CompilationArtifact {
            file_name: relative_path_str.to_string(),
            file_content,
            is_contract,
        });
    }

    // calling here explicitly to avoid dropping the AutoCleanUp struct
    auto_clean_up.clean_up().await;
    Ok(file_contents)
}
