use rocket::http::Status;
use rocket::response::Responder;
use rocket::Request;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CompileResponse {
    pub status: String,
    pub message: String,
    pub file_content: Vec<CompiledFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CompiledFile {
    pub file_name: String,
    pub file_content: String,
    #[serde(default)]
    pub is_contract: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct VerifyResponse {
    pub status: String,
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CompilationConfig {
    pub version: String,
    #[serde(default)]
    pub user_libraries: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CompilationRequest {
    pub config: CompilationConfig,
    pub contracts: Vec<CompiledFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContentMap {
    pub file_name: String,
    pub file_content: String,
}

#[derive(Debug)]
pub enum ApiCommand {
    CompilerVersion,
    Compile(CompilationRequest),
    Verify {
        version: String,
        network: String,
        contract_address: String,
        path: PathBuf,
        inputs: Vec<String>,
    },
    #[allow(dead_code)]
    Shutdown,
}

#[derive(Debug)]
pub enum ApiCommandResult {
    CompilerVersion(String),
    Compile(CompileResponse),
    Verify(VerifyResponse),
    #[allow(dead_code)]
    Shutdown,
}

pub struct HealthCheckResponse(pub Result<(), &'static str>);

impl<'r, 'o: 'r> Responder<'r, 'o> for HealthCheckResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        match self.0 {
            Ok(_) => {
                Ok(rocket::response::status::Custom(Status { code: 200 }, "OK")
                    .respond_to(request)?)
            }
            Err(_) => Ok(rocket::response::status::Custom(
                Status { code: 500 },
                "Internal Server Error",
            )
            .respond_to(request)?),
        }
    }
}

impl HealthCheckResponse {
    pub fn ok() -> Self {
        HealthCheckResponse(Ok(()))
    }

    pub fn error(value: &'static str) -> Self {
        HealthCheckResponse(Err(value))
    }
}
