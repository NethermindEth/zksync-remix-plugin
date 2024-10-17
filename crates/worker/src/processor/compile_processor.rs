use anyhow::{anyhow, Context};
use aws_sdk_s3::presigning::PresigningConfig;
use futures::future::join_all;
use futures::StreamExt;
use std::path::Path;
use std::time::Duration;
use tracing::error;
use types::item::errors::ServerError;
use types::item::task_result::{ArtifactInfo, TaskFailure, TaskResult, TaskSuccess};
use types::{CompilationRequest, ARTIFACTS_FOLDER};
use uuid::Uuid;

use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::commands::compile::{do_compile, ArtifactData, CompilationOutput};
use crate::processor::errors::CompileProcessorError;
use crate::processor::input_preparator::CompileInputPreparator;
use crate::purgatory::Purgatory;
use crate::utils::cleaner::AutoCleanUp;
use crate::utils::lib::{s3_compilation_files_dir, ZKSOLC_VERSIONS};

pub struct CompileProcessor {
    sqs_client: SqsClientWrapper,
    s3_client: S3ClientWrapper,
    input_preparator: CompileInputPreparator,
    purgatory: Purgatory,
}

impl CompileProcessor {
    pub fn new(
        sqs_client: SqsClientWrapper,
        s3_client: S3ClientWrapper,
        purgatory: Purgatory,
    ) -> Self {
        let input_preparator = CompileInputPreparator::new(s3_client.clone());
        Self {
            sqs_client,
            s3_client,
            input_preparator,
            purgatory,
        }
    }

    async fn validate_message(
        &self,
        message: &CompilationRequest,
    ) -> Result<(), CompileProcessorError> {
        let zksolc_version = message.config.version.as_str();
        if !ZKSOLC_VERSIONS.contains(&zksolc_version) {
            Err(CompileProcessorError::VersionNotSupportedError(
                zksolc_version.to_string(),
            ))
        } else {
            Ok(())
        }
    }

    // TODO(future me): could return bool.
    pub async fn process_message(
        &self,
        message: CompilationRequest,
        receipt_handle: String,
    ) -> Result<Vec<ArtifactInfo>, CompileProcessorError> {
        let id = message.id;

        self.validate_message(&message).await.map_err(|err| {
            // Reckoned as independent piece
            let receipt_handle_copy = receipt_handle.clone();
            let dir = s3_compilation_files_dir(id.to_string().as_str());
            let s3_client = self.s3_client.clone();
            let sqs_client = self.sqs_client.clone();
            tokio::spawn(async move {
                if let Err(err) = s3_client.delete_dir(&dir).await {
                    error!("Couldn't delete compilation files on failed validation: {err}")
                }
                if let Err(err) = sqs_client.delete_message(receipt_handle_copy).await {
                    error!("Failed to delete message from sqs: {err}");
                }
            });

            err
        })?;

        let compilation_input = self
            .input_preparator
            .prepare_input(&message)
            .await
            .map_err(|err| {
                let receipt_handle_copy = receipt_handle.clone();
                let sqs_client = self.sqs_client.clone();
                tokio::spawn(async move {
                    if let Err(err) = sqs_client.delete_message(receipt_handle_copy).await {
                        error!("Failed to delete message from sqs: {err}");
                    }
                });

                err
            })?;

        let compilation_output = do_compile(compilation_input).await?;
        match self
            .handle_output(id, &compilation_output, receipt_handle)
            .await
        {
            Ok(artifact_pairs) => {
                self.purgatory
                    .add_record(
                        id,
                        TaskResult::Success(TaskSuccess::Compile {
                            artifacts_info: artifact_pairs.clone(),
                        }),
                    )
                    .await;

                Ok(artifact_pairs)
            }
            Err(err) => {
                let task_result = TaskResult::Failure(TaskFailure {
                    error_type: ServerError::InternalError,
                    message: err.to_string(),
                });
                self.purgatory.add_record(id, task_result).await;

                Err(err.into())
            }
        }
    }

    async fn handle_output(
        &self,
        id: Uuid,
        compilation_output: &CompilationOutput,
        receipt_handle: String,
    ) -> anyhow::Result<Vec<ArtifactInfo>> {
        let auto_clean_up = AutoCleanUp {
            dirs: vec![compilation_output.workspace_dir.as_path()],
        };

        let file_keys = self.upload_artifacts(id, &compilation_output).await?;

        // Reckoned as independent piece
        {
            let dir = s3_compilation_files_dir(id.to_string().as_str());
            let s3_client = self.s3_client.clone();
            let sqs_client = self.sqs_client.clone();
            tokio::spawn(async move {
                // Clean compilation input files right away
                if let Err(err) = s3_client.delete_dir(&dir).await {
                    error!("Failed to delete compilation file: {err}")
                }
                if let Err(err) = sqs_client.delete_message(receipt_handle).await {
                    error!("Failed to delete message from sqs: {err}");
                }
            });
        }

        auto_clean_up.clean_up().await;
        Ok(self
            .generate_artifact_pairs(&file_keys, compilation_output.artifacts_data.as_slice())
            .await?)
    }

    async fn upload_artifact(
        &self,
        id: Uuid,
        artifacts_dir: &Path,
        el: &ArtifactData,
    ) -> anyhow::Result<String> {
        let absolute_path = artifacts_dir.join(&el.file_path);
        let file_content = tokio::fs::File::open(absolute_path.clone())
            .await
            .map_err(anyhow::Error::from)
            .with_context(|| format!("Couldn't open file: {}", absolute_path.display()))?;

        let file_key = format!(
            "{}/{}/{}",
            ARTIFACTS_FOLDER,
            id,
            el.file_path
                .to_str()
                .ok_or(anyhow!("Couldn't convert file_path to utf-8"))?
        );
        self.s3_client
            .put_object(&file_key, file_content)
            .await
            .map_err(anyhow::Error::from)
            .with_context(|| "Couldn't upload artifact")?; // TODO: TODO(101)

        Ok(file_key)
    }

    async fn upload_artifacts(
        &self,
        id: Uuid,
        compilation_output: &CompilationOutput,
    ) -> anyhow::Result<Vec<String>> {
        let upload_futures = compilation_output
            .artifacts_data
            .iter()
            .map(|el| self.upload_artifact(id, &compilation_output.artifacts_dir, el))
            .collect::<Vec<_>>();
        let results = join_all(upload_futures).await;
        let file_keys: Vec<String> = results.into_iter().collect::<Result<Vec<_>, _>>()?;

        Ok(file_keys)
    }

    async fn generate_artifact_pairs(
        &self,
        file_keys: &[String],
        artifact_paths: &[ArtifactData],
    ) -> anyhow::Result<Vec<ArtifactInfo>> {
        const DOWNLOAD_URL_EXPIRATION: Duration = Duration::from_secs(5 * 60 * 60);

        let mut artifact_pairs = Vec::with_capacity(file_keys.len());
        for (file_key, artifact_data) in file_keys.iter().zip(artifact_paths.iter()) {
            let expires_in = PresigningConfig::expires_in(DOWNLOAD_URL_EXPIRATION).unwrap();
            let presigned_request = self
                .s3_client
                .get_object_presigned(file_key.as_str(), &expires_in)
                .await
                .map_err(anyhow::Error::from)?; // TODO: maybe extra handle in case chan closed TODO(101)

            artifact_pairs.push(ArtifactInfo {
                artifact_type: artifact_data.artifact_type,
                file_path: artifact_data
                    .file_path
                    .to_str()
                    .ok_or(anyhow!(format!(
                        "Can't convert artifact_path to utf-8: {}",
                        artifact_data.file_path.display()
                    )))?
                    .to_string(),
                presigned_url: presigned_request.uri().to_string(),
            });
        }

        Ok(artifact_pairs)
    }
}
