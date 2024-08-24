use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

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

#[derive(Debug, Clone, Serialize)]
pub enum Status {
    // TODO: add FilesUploaded(?)
    Pending,
    Compiling,
    Ready(String),
    Failed(String),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "Pending"),
            Status::Compiling => write!(f, "Compiling"),
            Status::Ready(msg) => write!(f, "Ready: {}", msg),
            Status::Failed(msg) => write!(f, "Failed: {}", msg),
        }
    }
}

impl From<&Status> for u32 {
    fn from(value: &Status) -> Self {
        match value {
            Status::Pending => 0,
            Status::Compiling => 1,
            Status::Ready(_) => 2,
            Status::Failed(_) => 3,
        }
    }
}

impl From<Status> for HashMap<String, AttributeValue> {
    fn from(value: Status) -> Self {
        match value.clone() {
            Status::Pending | Status::Compiling => HashMap::from([(
                "Status".into(),
                AttributeValue::N(u32::from(&value).to_string()),
            )]),
            Status::Ready(val) | Status::Failed(val) => HashMap::from([
                (
                    "Status".into(),
                    AttributeValue::N(u32::from(&value).to_string()),
                ),
                ("Data".into(), AttributeValue::S(val)),
            ]),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ItemError {
    #[error("Invalid Item format")]
    FormatError,
    #[error(transparent)]
    ParseError(#[from] std::num::ParseIntError),
}

pub struct Item {
    // TODO: uuid?
    pub id: String,
    pub status: Status,
}

impl From<Item> for HashMap<String, AttributeValue> {
    fn from(value: Item) -> Self {
        let mut item_map = HashMap::from([("ID".into(), AttributeValue::S(value.id))]);
        item_map.extend(HashMap::from(value.status));

        item_map
    }
}

impl TryFrom<&HashMap<String, AttributeValue>> for Status {
    type Error = ItemError;
    fn try_from(value: &HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let status = value.get("Status").ok_or(ItemError::FormatError)?;
        let status: u32 = status
            .as_n()
            .map_err(|_| ItemError::FormatError)?
            .parse::<u32>()?;
        let status = match status {
            0 => Status::Pending,
            1 => Status::Compiling,
            2 => {
                let data = value.get("Data").ok_or(ItemError::FormatError)?;
                let data = data.as_s().map_err(|_| ItemError::FormatError)?;

                Status::Ready(data.clone())
            }
            3 => {
                let data = value.get("Data").ok_or(ItemError::FormatError)?;
                let data = data.as_s().map_err(|_| ItemError::FormatError)?;

                Status::Failed(data.clone())
            }
            _ => return Err(ItemError::FormatError),
        };

        Ok(status)
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for Item {
    type Error = ItemError;
    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Item, Self::Error> {
        let id = value.get("ID").ok_or(ItemError::FormatError)?;
        let id = id.as_s().map_err(|_| ItemError::FormatError)?;
        let status = (&value).try_into()?;

        Ok(Item {
            id: id.clone(),
            status,
        })
    }
}
