use aws_sdk_s3::types::Object;
use std::path::Path;
use std::process::Stdio;
use tracing::{instrument, warn};
use tracing::{error, info};
use uuid::Uuid;
use std::io::BufWriter;
use std::io::{Write, IoSlice};
use std::ops::Add;
use tokio::io::AsyncWriteExt;

use crate::dynamodb_client::DynamoDBClient;
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::hardhat_config::HardhatConfigBuilder;
use crate::utils::lib::{
    generate_folder_name, status_code_to_message,
    DEFAULT_SOLIDITY_VERSION, SOL_ROOT, ZKSOLC_VERSIONS,
};

async fn list_all_keys(
    client: &aws_sdk_s3::Client,
    id: String,
    bucket: &str,
) -> Result<Vec<Object>, ()> {
    // TODO: errors
    let mut objects = Vec::new();
    let mut continuation_token: Option<String> = None;

    let id = id.clone().add("/");
    loop {
        let mut request = client
            .list_objects_v2()
            .bucket(bucket)
            .delimiter('/')
            .prefix(id.clone());
        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }

        let response = request.send().await.map_err(|_| ())?;
        if let Some(contents) = response.contents {
            objects.extend(contents);
        }

        let is_truncated = if let Some(is_truncated) = response.is_truncated {
            is_truncated
        } else {
            warn!("is_truncated empty");
            break;
        };

        if !is_truncated {
            break;
        }

        continuation_token = response.next_continuation_token;
        if continuation_token.is_none() {
            error!("continuation_token wasn't set!");
            break;
        }
    }

    Ok(objects)
}

pub async fn compile(
    id: Uuid,
    db_client: DynamoDBClient,
    s3_client: aws_sdk_s3::Client,
) -> Result<(), ()> {
    // TODO: errors
    let item = db_client.get_item(id.to_string()).await.unwrap();
    let item = match item {
        Some(item) => item,
        None => {
            error!("No item id: {}", id);
            return Err(());
        }
    };

    let objects = list_all_keys(&s3_client, id.to_string(), "TODO").await?;
    for object in objects {
        let key = object.key().ok_or(())?;
        let mut object = s3_client
            .get_object()
            .bucket("TODO")
            .key(key)
            .send()
            .await
            .map_err(|_| ())?;

        let mut byte_count = 0_usize;
        let mut contents = Vec::new();
        while let Some(bytes) = object.body.try_next().await.map_err(|_| ())? {
            let bytes_len = bytes.len();
            std::io::Write::write_all(&mut contents, &bytes).map_err(|_| ());
            byte_count += bytes_len;
        }
    }

    // TODO:
    Ok(())
}
//
// pub async fn do_compile(
//     id: Uuid,
//     db_client: DynamoDBClient,
//     s3_client: aws_sdk_s3::Client,
//     compilation_request: CompilationRequest,
// ) -> Result<Json<CompileResponse>> {
//     // TODO: errors
//
//     let zksolc_version = compilation_request.config.version;
//
//     // check if the version is supported
//     if !ZKSOLC_VERSIONS.contains(&zksolc_version.as_str()) {
//         return Err(ApiError::VersionNotSupported(zksolc_version));
//     }
//
//     if compilation_request.contracts.is_empty() {
//         return Ok(Json(CompileResponse {
//             file_content: vec![],
//             status: status_code_to_message(Some(0)),
//             message: "Nothing to compile".into(),
//         }));
//     }
//
//     let namespace = generate_folder_name();
//
//     // root directory for the contracts
//     let workspace_path_str = format!("{}/{}", SOL_ROOT, namespace);
//     let workspace_path = Path::new(&workspace_path_str);
//
//     // root directory for the artifacts
//     let artifacts_path_str = format!("{}/{}", workspace_path_str, "artifacts-zk");
//     let artifacts_path = Path::new(&artifacts_path_str);
//
//     // root directory for user files (hardhat config, etc)
//     let user_files_path_str = workspace_path_str.clone();
//     let hardhat_config_path = Path::new(&user_files_path_str).join("hardhat.config.ts");
//
//     // instantly create the directories
//     tokio::fs::create_dir_all(workspace_path)
//         .await
//         .map_err(ApiError::FailedToWriteFile)?;
//     tokio::fs::create_dir_all(artifacts_path)
//         .await
//         .map_err(ApiError::FailedToWriteFile)?;
//
//     // when the compilation is done, clean up the directories
//     // it will be called when the AutoCleanUp struct is dropped
//     let auto_clean_up = AutoCleanUp {
//         dirs: vec![workspace_path.to_str().unwrap()],
//     };
//
//     // write the hardhat config file
//     let mut hardhat_config_builder = HardhatConfigBuilder::new();
//     hardhat_config_builder
//         .zksolc_version(&zksolc_version)
//         .solidity_version(DEFAULT_SOLIDITY_VERSION);
//     if let Some(target_path) = compilation_request.target_path {
//         hardhat_config_builder.paths_sources(&target_path);
//     }
//
//     let hardhat_config_content = hardhat_config_builder.build().to_string_config();
//
//     // create parent directories
//     tokio::fs::create_dir_all(hardhat_config_path.parent().unwrap())
//         .await
//         .map_err(ApiError::FailedToWriteFile)?;
//
//     tokio::fs::write(hardhat_config_path, hardhat_config_content)
//         .await
//         .map_err(ApiError::FailedToWriteFile)?;
//
//     // filter test files from compilation candidates
//     let contracts = compilation_request
//         .contracts
//         .into_iter()
//         .filter(|contract| !contract.file_name.ends_with("_test.sol"))
//         .collect();
//
//     // initialize the files
//     initialize_files(contracts, workspace_path).await?;
//
//     // Limit number of spawned processes. RAII released
//     let _permit = SPAWN_SEMAPHORE.acquire().await.expect("Expired semaphore");
//
//     let command = tokio::process::Command::new("npx")
//         .arg("hardhat")
//         .arg("compile")
//         .current_dir(workspace_path)
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped())
//         .spawn();
//     let process = command.map_err(ApiError::FailedToExecuteCommand)?;
//     let output = process
//         .wait_with_output()
//         .await
//         .map_err(ApiError::FailedToReadOutput)?;
//
//     let status = output.status;
//     let message = String::from_utf8_lossy(&output.stdout).to_string();
//
//     info!("Output: \n{:?}", String::from_utf8_lossy(&output.stdout));
//     if !status.success() {
//         error!(
//             "Compilation error: {}",
//             String::from_utf8_lossy(&output.stderr)
//         );
//         return Ok(Json(CompileResponse {
//             file_content: vec![],
//             message: format!(
//                 "Failed to compile:\n{}",
//                 String::from_utf8_lossy(&output.stderr)
//             ),
//             status: "Error".to_string(),
//         }));
//     }
//
//     // fetch the files in the artifacts directory
//     let mut file_contents: Vec<CompiledFile> = vec![];
//     let file_paths = list_files_in_directory(artifacts_path);
//
//     for file_path in file_paths.iter() {
//         let file_content = tokio::fs::read_to_string(file_path)
//             .await
//             .map_err(ApiError::FailedToReadFile)?;
//         let full_path = Path::new(file_path);
//         let relative_path = full_path.strip_prefix(artifacts_path).unwrap_or(full_path);
//         let relative_path_str = relative_path.to_str().unwrap();
//
//         // todo(varex83): is it the best way to check?
//         let is_contract =
//             !relative_path_str.ends_with(".dbg.json") && relative_path_str.ends_with(".json");
//
//         file_contents.push(CompiledFile {
//             file_name: relative_path_str.to_string(),
//             file_content,
//             is_contract,
//         });
//     }
//
//     // calling here explicitly to avoid dropping the AutoCleanUp struct
//     auto_clean_up.clean_up().await;
//
//     Ok(Json(CompileResponse {
//         file_content: file_contents,
//         status: status_code_to_message(status.code()),
//         message,
//     }))
// }
