use std::path::Path;
use types::CompilationRequest;

use crate::clients::dynamodb_clients::wrapper::DynamoDBClientWrapper;
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::commands::compile::CompilationInput;
use crate::commands::errors::PreparationError;
use crate::utils::lib::{SOL_ROOT, ZKSOLC_VERSIONS};

pub struct InputPreparator {
    s3_client: S3ClientWrapper,
}

impl InputPreparator {
    pub fn new(s3_client: S3ClientWrapper) -> Self {
        Self {
            s3_client,
        }
    }

    pub(crate) async fn prepare_compile_input(
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
