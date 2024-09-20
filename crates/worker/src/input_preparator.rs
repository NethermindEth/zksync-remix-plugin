use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use std::path::Path;
use tracing::{error, warn};
use types::item::{Item, Status};
use types::CompilationRequest;
use uuid::Uuid;

use crate::clients::dynamodb_clients::wrapper::DynamoDBClientWrapper;
use crate::clients::errors::DBError;
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::commands::compile::CompilationInput;
use crate::commands::errors::PreparationError;
use crate::utils::lib::{SOL_ROOT, ZKSOLC_VERSIONS};

pub struct InputPreparator<'a> {
    db_client: &'a DynamoDBClientWrapper,
    s3_client: &'a S3ClientWrapper,
}

impl<'a> InputPreparator<'a> {
    pub fn new(db_client: &'a DynamoDBClientWrapper, s3_client: &'a S3ClientWrapper) -> Self {
        Self {
            db_client,
            s3_client,
        }
    }

    pub(crate) async fn prepare_compile_input(
        &self,
        request: &CompilationRequest,
    ) -> Result<CompilationInput, PreparationError> {
        let zksolc_version = request.config.version.as_str();
        if !ZKSOLC_VERSIONS.contains(&zksolc_version) {
            return Err(PreparationError::VersionNotSupported(
                zksolc_version.to_string(),
            ));
        }

        let item = self
            .db_client
            .get_item(request.id.to_string().as_str())
            .await?;
        let item: Item = match item {
            Some(item) => item,
            None => {
                return Err(PreparationError::NoDBItemError(request.id.to_string()));
            }
        };

        if !matches!(item.status, Status::Pending) {
            warn!("Item already processing: {}", item.status);
            return Err(PreparationError::UnexpectedStatusError(
                item.status.to_string(),
            ));
        }

        let dir = format!("{}/", request.id);
        let files = self.s3_client.extract_files(&dir).await?;

        // Update status to Compiling
        self.try_set_compiling_status(request.id).await?;

        Ok(CompilationInput {
            workspace_path: Path::new(SOL_ROOT).join(request.id.to_string().as_str()),
            config: request.config.clone(),
            contracts: files,
        })
    }

    async fn try_set_compiling_status(&self, key: Uuid) -> Result<(), PreparationError> {
        let db_update_result = self
            .db_client
            .update_item_status_conditional(
                key.to_string().as_str(),
                &Status::Pending,
                &Status::InProgress,
            )
            .await;
        match db_update_result {
            Ok(_) => Ok(()),
            Err(SdkError::ServiceError(err)) => match err.err() {
                UpdateItemError::ConditionalCheckFailedException(_) => {
                    error!("Conditional check not met");
                    Err(PreparationError::UnexpectedStatusError(
                        "Concurrent status change from another instance".into(),
                    ))
                }
                _ => Err(DBError::from(SdkError::ServiceError(err)).into()),
            },
            Err(err) => Err(DBError::from(err).into()),
        }
    }
}
