use anyhow::Context;
use std::path::PathBuf;
use std::process::Stdio;
use types::VerifyConfig;

use crate::commands::errors::VerificationError;
use crate::commands::{CompilationFile, SPAWN_SEMAPHORE};
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{initialize_files, ALLOWED_NETWORKS, DEFAULT_SOLIDITY_VERSION};

pub struct VerificationInput {
    pub workspace_path: PathBuf,
    pub config: VerifyConfig,
    pub contracts: Vec<CompilationFile>,
}

fn extract_verify_args(config: &VerifyConfig) -> Vec<String> {
    let VerifyConfig {
        target_contract,
        network,
        contract_address,
        inputs,
        ..
    } = &config;

    let mut args: Vec<String> = vec!["hardhat".into(), "verify".into(), "--network".into()];
    if network == "sepolia" {
        args.push("zkSyncTestnet".into())
    } else {
        args.push("zkSyncMainnet".into())
    }

    if let Some(target_contract) = target_contract {
        args.push("--contract".into());
        args.push(target_contract.clone());
    }

    args.push(contract_address.clone());
    args.extend(inputs.clone());

    args
}

pub async fn do_verify(
    verification_request: VerificationInput,
) -> Result<String, VerificationError> {
    let zksolc_version = verification_request.config.zksolc_version.clone();

    let solc_version = verification_request
        .config
        .solc_version
        .clone()
        .unwrap_or(DEFAULT_SOLIDITY_VERSION.to_string());

    let network = verification_request.config.network.clone();

    // check if the network is supported
    if !ALLOWED_NETWORKS.contains(&network.as_str()) {
        return Err(VerificationError::UnknownNetworkError(network));
    }

    // root directory for the contracts
    let workspace_path = verification_request.workspace_path;
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

    // when the compilation is done, clean up the directories
    // it will be called when the AutoCleanUp struct is dropped
    let auto_clean_up = AutoCleanUp {
        dirs: vec![workspace_path.as_path()],
    };

    // write the hardhat config file
    let hardhat_config_content = HardhatConfigBuilder::new()
        .zksolc_version(&zksolc_version)
        .solidity_version(&solc_version)
        .build()
        .to_string_config();

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

    // initialize the files
    initialize_files(&workspace_path, verification_request.contracts)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Couldn't write contract to fs")?;

    // Limit number of spawned processes. RAII released
    let _permit = SPAWN_SEMAPHORE.acquire().await.expect("Expired semaphore");

    let args = extract_verify_args(&verification_request.config);
    let process = tokio::process::Command::new("npx")
        .args(args)
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
    let status = output.status;
    let message = String::from_utf8_lossy(&output.stdout).to_string();

    // calling here explicitly to avoid dropping the AutoCleanUp struct
    auto_clean_up.clean_up().await;

    if !status.success() {
        Err(VerificationError::VerificationFailureError(message))
    } else {
        Ok(message)
    }
}
