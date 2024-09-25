pub mod item;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ARTIFACTS_FOLDER: &str = "artifacts";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompilationConfig {
    pub version: String,
    #[serde(default)]
    pub user_libraries: Vec<String>,
    // TODO: reflect change in UI-code
    pub target_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompilationRequest {
    pub id: Uuid,
    pub config: CompilationConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerifyConfig {
    pub zksolc_version: String,
    pub solc_version: Option<String>,
    pub network: String,
    pub contract_address: String,
    pub inputs: Vec<String>,
    pub target_contract: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerificationRequest {
    pub id: Uuid,
    pub config: VerifyConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SqsMessage {
    Compile {
        #[serde(flatten)]
        request: CompilationRequest,
    },
    Verify {
        #[serde(flatten)]
        request: VerificationRequest,
    },
}

impl SqsMessage {
    pub fn id(&self) -> Uuid {
        match self {
            SqsMessage::Compile { request } => request.id,
            SqsMessage::Verify { request } => request.id,
        }
    }
}


#[derive(thiserror::Error, Debug)]
pub enum SqsRawMessageError {
    #[error("Empty message body")]
    NoMessageBody,
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error)
}

impl TryFrom<aws_sdk_sqs::types::Message> for SqsMessage {
    type Error = SqsRawMessageError;

    fn try_from(value: aws_sdk_sqs::types::Message) -> Result<Self, Self::Error> {
        let body = value.body.ok_or(SqsRawMessageError::NoMessageBody)?;
        let sqs_message = serde_json::from_str::<SqsMessage>(&body)?;
        Ok(sqs_message)
    }
}