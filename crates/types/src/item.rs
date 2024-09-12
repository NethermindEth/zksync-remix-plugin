use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use uuid::Uuid;

pub type AttributeMap = HashMap<String, AttributeValue>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    Success { presigned_urls: Vec<String> },
    Failure(String),
}

#[derive(Debug, Clone)]
pub enum Status {
    // TODO: add FilesUploaded(?)
    Pending,
    InProgress,
    Done(TaskResult),
}

impl Status {
    pub const fn attribute_name() -> &'static str {
        "Status"
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "Pending"),
            Status::InProgress => write!(f, "InProgress"),
            Status::Done(TaskResult::Success { .. }) => write!(f, "Success"),
            Status::Done(TaskResult::Failure(msg)) => write!(f, "Failure: {}", msg),
        }
    }
}

impl From<&Status> for u32 {
    fn from(value: &Status) -> Self {
        match value {
            Status::Pending => 0,
            Status::InProgress => 1,
            Status::Done(TaskResult::Success { .. }) => 2,
            Status::Done(TaskResult::Failure(_)) => 3,
        }
    }
}

impl From<Status> for u32 {
    fn from(value: Status) -> Self {
        u32::from(&value)
    }
}

impl From<TaskResult> for AttributeMap {
    fn from(value: TaskResult) -> Self {
        match value {
            TaskResult::Success { presigned_urls } => HashMap::from([(
                Item::data_attribute_name().into(),
                AttributeValue::Ss(presigned_urls),
            )]),
            TaskResult::Failure(message) => HashMap::from([(
                Item::data_attribute_name().into(),
                AttributeValue::S(message),
            )]),
        }
    }
}

impl From<Status> for AttributeMap {
    fn from(value: Status) -> Self {
        let mut map = HashMap::from([(
            Status::attribute_name().into(),
            AttributeValue::N(u32::from(&value).to_string()),
        )]);

        // For `Done` variant, reuse the conversion logic from `TaskResult`
        if let Status::Done(task_result) = value {
            map.extend(AttributeMap::from(task_result));
        }

        map
    }
}

impl TryFrom<&AttributeMap> for Status {
    type Error = ItemError;
    fn try_from(value: &AttributeMap) -> Result<Self, Self::Error> {
        let status = value
            .get(Status::attribute_name())
            .ok_or(ItemError::FormatError)?;
        let status: u32 = status
            .as_n()
            .map_err(|_| ItemError::FormatError)?
            .parse::<u32>()?;
        let status = match status {
            0 => Status::Pending,
            1 => Status::InProgress,
            2 => {
                let data = value
                    .get(Item::data_attribute_name())
                    .ok_or(ItemError::FormatError)?;
                let data = data.as_ss().map_err(|_| ItemError::FormatError)?;

                Status::Done(TaskResult::Success {
                    presigned_urls: data.clone(),
                })
            }
            3 => {
                let data = value
                    .get(Item::data_attribute_name())
                    .ok_or(ItemError::FormatError)?;
                let data = data.as_s().map_err(|_| ItemError::FormatError)?;

                Status::Done(TaskResult::Failure(data.clone()))
            }
            _ => return Err(ItemError::FormatError),
        };

        Ok(status)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ItemError {
    #[error("Invalid Item format")]
    FormatError,
    #[error(transparent)]
    NumParseError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    DataParseError(#[from] chrono::format::ParseError),
}

pub struct Item {
    pub id: Uuid,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    // TODO: type: Compiling/Verifying
}

impl Item {
    pub const fn status_attribute_name() -> &'static str {
        Status::attribute_name()
    }

    pub const fn data_attribute_name() -> &'static str {
        "Data"
    }

    pub const fn id_attribute_name() -> &'static str {
        "ID"
    }

    pub const fn created_at_attribute_name() -> &'static str {
        "CreatedAt"
    }

    pub const fn primary_key_name() -> &'static str {
        Self::id_attribute_name()
    }
}

impl From<Item> for AttributeMap {
    fn from(value: Item) -> Self {
        let mut item_map = HashMap::from([
            (
                Item::id_attribute_name().into(),
                AttributeValue::S(value.id.into()),
            ),
            (
                Item::created_at_attribute_name().into(),
                AttributeValue::S(value.created_at.to_rfc3339()),
            ),
        ]);
        item_map.extend(HashMap::from(value.status));

        item_map
    }
}

impl TryFrom<AttributeMap> for Item {
    type Error = ItemError;
    fn try_from(value: AttributeMap) -> Result<Item, Self::Error> {
        let id = value
            .get(Item::id_attribute_name())
            .ok_or(ItemError::FormatError)?
            .as_s()
            .map_err(|_| ItemError::FormatError)?;
        let id = Uuid::parse_str(id.as_str()).map_err(|_| ItemError::FormatError)?;
        let status = (&value).try_into()?;

        let created_at = value
            .get(Item::created_at_attribute_name())
            .ok_or(ItemError::FormatError)?;
        let created_at = created_at.as_s().map_err(|_| ItemError::FormatError)?;
        let created_at = DateTime::<FixedOffset>::parse_from_rfc3339(created_at.as_str())?;

        Ok(Item {
            id,
            status,
            created_at: created_at.into(),
        })
    }
}
