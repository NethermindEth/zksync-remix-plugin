use std::path::Path;
use types::{CompilationRequest, VerificationRequest};

use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::commands::compile::CompilationInput;
use crate::commands::verify::VerificationInput;
use crate::utils::lib::SOL_ROOT;

pub struct CompileInputPreparator {
    s3_client: S3ClientWrapper,
}

impl CompileInputPreparator {
    pub fn new(s3_client: S3ClientWrapper) -> Self {
        Self { s3_client }
    }

    pub(crate) async fn prepare_input(
        &self,
        request: &CompilationRequest,
    ) -> anyhow::Result<CompilationInput> {
        let dir = format!("{}/", request.id);
        let files = self
            .s3_client
            .extract_files(&dir)
            .await
            .map_err(anyhow::Error::from)?;

        Ok(CompilationInput {
            workspace_path: Path::new(SOL_ROOT).join(request.id.to_string().as_str()),
            config: request.config.clone(),
            contracts: files,
        })
    }
}

pub struct VerifyInputPreparator {
    s3_client: S3ClientWrapper,
}

impl VerifyInputPreparator {
    pub fn new(s3_client: S3ClientWrapper) -> Self {
        Self { s3_client }
    }

    pub(crate) async fn prepare_input(
        &self,
        request: &VerificationRequest,
    ) -> anyhow::Result<VerificationInput> {
        let dir = format!("{}/", request.id);
        let files = self
            .s3_client
            .extract_files(&dir)
            .await
            .map_err(anyhow::Error::from)?;

        Ok(VerificationInput {
            workspace_path: Path::new(SOL_ROOT).join(request.id.to_string().as_str()),
            config: request.config.clone(),
            contracts: files,
        })
    }
}
