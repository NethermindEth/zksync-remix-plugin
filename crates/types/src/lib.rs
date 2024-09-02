pub mod item;

use serde::{Deserialize, Serialize};

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
    pub id: String,
    pub config: CompilationConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerifyConfig {
    pub zksolc_version: String,
    pub solc_version: Option<String>,
    pub network: String,
    pub contract_address: String,
    pub inputs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerificationRequest {
    pub id: String,
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
