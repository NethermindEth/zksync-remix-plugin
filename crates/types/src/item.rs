use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, FixedOffset, Utc};
use errors::ItemError;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use uuid::Uuid;

use crate::item::task_result::TaskResult;

pub mod errors;
pub mod task_result;

pub type AttributeMap = HashMap<String, AttributeValue>;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Status {
    // TODO: add FilesUploaded(?)
    Pending,
    InProgress,
    Done(TaskResult), // TODO: TaskResult will be generic probably
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
            Status::Done(val) => val.fmt(f),
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

impl From<Status> for AttributeMap {
    fn from(value: Status) -> Self {
        let mut map = HashMap::from([(
            Status::attribute_name().into(),
            AttributeValue::N(u32::from(&value).to_string()),
        )]);

        // For `Done` variant, reuse the conversion logic from `TaskResult`
        if let Status::Done(task_result) = value {
            let task_result_map = HashMap::from([(
                TaskResult::attribute_name().into(),
                AttributeValue::M(AttributeMap::from(task_result)),
            )]);
            map.extend(task_result_map);
        }

        map
    }
}

impl TryFrom<&AttributeMap> for Status {
    type Error = ItemError;
    fn try_from(value: &AttributeMap) -> Result<Self, Self::Error> {
        let status = value
            .get(Status::attribute_name())
            .ok_or(ItemError::absent_attribute_error(Status::attribute_name()))?;
        let status: u32 = status
            .as_n()
            .map_err(|_| ItemError::invalid_attribute_type(Status::attribute_name(), "number"))?
            .parse::<u32>()?;

        match status {
            0 => Ok(Status::Pending),
            1 => Ok(Status::InProgress),
            2 | 3 => {
                let data = value.get(TaskResult::attribute_name()).ok_or(
                    ItemError::absent_attribute_error(TaskResult::attribute_name()),
                )?;
                let data = data.as_m().map_err(|_| {
                    ItemError::invalid_attribute_type(TaskResult::attribute_name(), "map")
                })?;

                let task_result: TaskResult = data.try_into()?;
                match (status, &task_result) {
                    (2, TaskResult::Success(_)) => Ok(Status::Done(task_result)),
                    (3, TaskResult::Failure(_)) => Ok(Status::Done(task_result)),
                    _ => Err(ItemError::FormatError(format!(
                        "status is {}, while actual value: {}",
                        status, task_result
                    ))),
                }
            }
            val => return Err(ItemError::FormatError(format!("Status value is: {}", val))),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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
            .ok_or(ItemError::absent_attribute_error(Item::id_attribute_name()))?
            .as_s()
            .map_err(|_| ItemError::invalid_attribute_type(Item::id_attribute_name(), "string"))?;
        let id =
            Uuid::parse_str(id.as_str()).map_err(|err| ItemError::FormatError(err.to_string()))?;
        let status = (&value).try_into()?;

        let created_at = value.get(Item::created_at_attribute_name()).ok_or(
            ItemError::absent_attribute_error(Item::created_at_attribute_name()),
        )?;
        let created_at = created_at.as_s().map_err(|_| {
            ItemError::invalid_attribute_type(Item::created_at_attribute_name(), "string")
        })?;
        let created_at = DateTime::<FixedOffset>::parse_from_rfc3339(created_at.as_str())?;

        Ok(Item {
            id,
            status,
            created_at: created_at.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{task_result::*, *};

    use aws_sdk_dynamodb::types::AttributeValue;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    use crate::item::errors::ServerError;
    use crate::item::task_result::tests::{task_success_compile, task_success_compile_map};

    #[test]
    fn test_status_pending_to_attribute_map() {
        let status = Status::Pending;
        let expected_map = HashMap::from([(
            Status::attribute_name().to_string(),
            AttributeValue::N("0".to_string()),
        )]);

        let attribute_map: AttributeMap = status.into();
        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_status_pending_from_attribute_map() {
        let attribute_map = HashMap::from([(
            Status::attribute_name().to_string(),
            AttributeValue::N("0".to_string()),
        )]);

        let result: Status = (&attribute_map).try_into().expect("Conversion failed");
        assert_eq!(result, Status::Pending);
    }

    #[test]
    fn test_status_inprogress_to_attribute_map() {
        let status = Status::InProgress;
        let expected_map = HashMap::from([(
            Status::attribute_name().to_string(),
            AttributeValue::N("1".to_string()),
        )]);

        let attribute_map: AttributeMap = status.into();
        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_status_inprogress_from_attribute_map() {
        let attribute_map = HashMap::from([(
            Status::attribute_name().to_string(),
            AttributeValue::N("1".to_string()),
        )]);

        let result: Status = (&attribute_map).try_into().expect("Conversion failed");
        assert_eq!(result, Status::InProgress);
    }

    fn status_success_compile_map() -> AttributeMap {
        HashMap::from([
            (
                Status::attribute_name().to_string(),
                AttributeValue::N("2".to_string()),
            ),
            (
                TaskResult::attribute_name().to_string(),
                AttributeValue::M(HashMap::from([(
                    TaskResult::success_attribute_name().to_string(),
                    AttributeValue::M(task_success_compile_map()),
                )])),
            ),
        ])
    }

    #[test]
    fn test_status_done_compile_success_to_attribute_map() {
        let expected_map = status_success_compile_map();

        let task_result = TaskResult::Success(task_success_compile());
        let status = Status::Done(task_result.clone());
        let attribute_map: AttributeMap = status.into();

        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_status_done_compile_success_from_attribute_map() {
        let expected_result = Status::Done(TaskResult::Success(task_success_compile()));

        let attribute_map = status_success_compile_map();
        let result: Status = (&attribute_map).try_into().expect("Conversion failed");

        assert_eq!(result, expected_result);
    }

    fn status_failure_compilation_map() -> AttributeMap {
        HashMap::from([
            (
                Status::attribute_name().to_string(),
                AttributeValue::N("3".to_string()),
            ),
            (
                TaskResult::attribute_name().to_string(),
                AttributeValue::M(HashMap::from([(
                    TaskResult::failure_attribute_name().to_string(),
                    AttributeValue::Ss(vec![
                        "CompilationError".to_string(),
                        "Compilation failed".to_string(),
                    ]),
                )])),
            ),
        ])
    }

    #[test]
    fn test_status_done_failure_to_attribute_map() {
        let expected_map = status_failure_compilation_map();

        let task_result = TaskResult::Failure(TaskFailure {
            error_type: ServerError::CompilationError,
            message: "Compilation failed".to_string(),
        });
        let status = Status::Done(task_result);
        let attribute_map: AttributeMap = status.into();

        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_status_done_failure_from_attribute_map() {
        let expected_result = Status::Done(TaskResult::Failure(TaskFailure {
            error_type: ServerError::CompilationError,
            message: "Compilation failed".to_string(),
        }));

        let attribute_map = status_failure_compilation_map();
        let result: Status = (&attribute_map).try_into().expect("Conversion failed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_item_to_attribute_map() {
        let item = Item {
            id: Uuid::new_v4(),
            status: Status::InProgress,
            created_at: Utc::now(),
        };

        let expected_map = HashMap::from([
            (
                Item::id_attribute_name().to_string(),
                AttributeValue::S(item.id.to_string()),
            ),
            (
                Item::created_at_attribute_name().to_string(),
                AttributeValue::S(item.created_at.to_rfc3339()),
            ),
            (
                Status::attribute_name().to_string(),
                AttributeValue::N("1".to_string()),
            ),
        ]);

        let attribute_map: AttributeMap = item.clone().into();
        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_item_from_attribute_map() {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let attribute_map = HashMap::from([
            (
                Item::id_attribute_name().to_string(),
                AttributeValue::S(id.to_string()),
            ),
            (
                Item::created_at_attribute_name().to_string(),
                AttributeValue::S(created_at.to_rfc3339()),
            ),
            (
                Status::attribute_name().to_string(),
                AttributeValue::N("1".to_string()),
            ),
        ]);

        let expected_item = Item {
            id,
            status: Status::InProgress,
            created_at,
        };

        let result: Item = attribute_map.try_into().expect("Conversion failed");
        assert_eq!(result, expected_item);
    }
}
