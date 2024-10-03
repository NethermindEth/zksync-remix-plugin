use aws_sdk_dynamodb::types::AttributeValue;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Formatter;

use crate::item::errors::{ItemError, ServerError};
use crate::item::AttributeMap;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TaskResult {
    Success(TaskSuccess),
    Failure(TaskFailure),
}

impl TaskResult {
    pub const fn attribute_name() -> &'static str {
        "Data"
    }

    pub const fn success_attribute_name() -> &'static str {
        "Success"
    }

    pub const fn failure_attribute_name() -> &'static str {
        "Failure"
    }
}

impl std::fmt::Display for TaskResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskResult::Success(_) => write!(f, "Success"),
            TaskResult::Failure(msg) => write!(f, "Failure: {}", msg.message),
        }
    }
}

impl From<TaskResult> for AttributeMap {
    fn from(value: TaskResult) -> Self {
        match value {
            TaskResult::Success(task_success) => HashMap::from([(
                TaskResult::success_attribute_name().into(),
                AttributeValue::M(task_success.into()),
            )]),
            TaskResult::Failure(failure) => failure.into(),
        }
    }
}

impl TryFrom<&AttributeMap> for TaskResult {
    type Error = ItemError;
    fn try_from(value: &AttributeMap) -> Result<Self, Self::Error> {
        let (key, value) = if let Some(el) = value.iter().next() {
            if value.len() != 1 {
                Err(ItemError::FormatError(
                    "More than 1 key in TaskSuccess".into(),
                ))
            } else {
                Ok(el)
            }
        } else {
            Err(ItemError::FormatError("No keys for TaskResult".into()))
        }?;

        match key.as_str() {
            "Success" => {
                let data = value
                    .as_m()
                    .map_err(|_| ItemError::FormatError("invalid type".into()))?;
                Ok(TaskResult::Success(data.try_into()?))
            }
            "Failure" => Ok(TaskResult::Failure(value.try_into()?)),
            _ => Err(ItemError::FormatError(format!(
                "Invalid key in TaskResult: {key}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged)]
pub enum TaskSuccess {
    Compile { presigned_urls: Vec<String> },
    Verify { message: String },
}

impl TaskSuccess {
    pub const fn compile_attribute_name() -> &'static str {
        "Compile"
    }

    pub const fn verify_attribute_name() -> &'static str {
        "Verify"
    }
}

impl From<TaskSuccess> for AttributeMap {
    fn from(value: TaskSuccess) -> Self {
        match value {
            TaskSuccess::Compile { presigned_urls } => HashMap::from([(
                TaskSuccess::compile_attribute_name().into(),
                AttributeValue::Ss(presigned_urls),
            )]),
            TaskSuccess::Verify { message } => HashMap::from([(
                TaskSuccess::verify_attribute_name().into(),
                AttributeValue::S(message),
            )]),
        }
    }
}

impl TryFrom<&AttributeMap> for TaskSuccess {
    type Error = ItemError;

    fn try_from(value: &AttributeMap) -> Result<Self, Self::Error> {
        let (key, value) = if let Some(el) = value.iter().next() {
            if value.len() != 1 {
                Err(ItemError::FormatError(
                    "More than 1 key in TaskSuccess".into(),
                ))
            } else {
                Ok(el)
            }
        } else {
            Err(ItemError::FormatError("No keys for TaskResult".into()))
        }?;

        match key.as_str() {
            "Compile" => {
                let presigned_urls = value
                    .as_ss()
                    .map_err(|_| ItemError::FormatError("invalid type".into()))?;

                Ok(TaskSuccess::Compile {
                    presigned_urls: presigned_urls.clone(),
                })
            }
            "Verify" => {
                let message = value
                    .as_s()
                    .map_err(|_| ItemError::FormatError("invalid type".into()))?;

                Ok(TaskSuccess::Verify {
                    message: message.clone(),
                })
            }
            _ => Err(ItemError::FormatError(format!(
                "Invalid key in TaskSuccess: {key}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct TaskFailure {
    pub error_type: ServerError,
    pub message: String,
}

impl TaskFailure {
    pub const fn attribute_name() -> &'static str {
        "Failure"
    }
}

impl From<TaskFailure> for AttributeMap {
    fn from(value: TaskFailure) -> Self {
        HashMap::from([(
            TaskFailure::attribute_name().into(),
            AttributeValue::Ss(vec![
                <ServerError as Into<&'static str>>::into(value.error_type).to_string(),
                value.message,
            ]),
        )])
    }
}

impl TryFrom<&AttributeValue> for TaskFailure {
    type Error = ItemError;

    fn try_from(value: &AttributeValue) -> Result<Self, Self::Error> {
        let data = value
            .as_ss()
            .map_err(|_| ItemError::FormatError("invalid type".into()))?;
        if data.len() != 2 {
            Err(ItemError::FormatError(
                "Invalid Failure values format".into(),
            ))
        } else {
            let error_type: ServerError = data[0]
                .as_str()
                .try_into()
                .map_err(ItemError::FormatError)?;
            Ok(TaskFailure {
                error_type,
                message: data[1].to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_dynamodb::types::AttributeValue;
    use std::collections::HashMap;

    #[test]
    fn test_task_success_compile_to_attribute_map() {
        let task_success = TaskSuccess::Compile {
            presigned_urls: vec!["url1".to_string(), "url2".to_string()],
        };

        let expected_map = HashMap::from([(
            "Compile".to_string(),
            AttributeValue::Ss(vec!["url1".to_string(), "url2".to_string()]),
        )]);

        let attribute_map: AttributeMap = task_success.into();
        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_task_success_compile_from_attribute_map() {
        let attribute_map = HashMap::from([(
            "Compile".to_string(),
            AttributeValue::Ss(vec!["url1".to_string(), "url2".to_string()]),
        )]);

        let expected_task_success = TaskSuccess::Compile {
            presigned_urls: vec!["url1".to_string(), "url2".to_string()],
        };

        let result: TaskSuccess = (&attribute_map).try_into().expect("Conversion failed");
        assert_eq!(result, expected_task_success);
    }

    #[test]
    fn test_task_success_verify_to_attribute_map() {
        let task_success = TaskSuccess::Verify {
            message: "Verification successful".to_string(),
        };

        let expected_map = HashMap::from([(
            "Verify".to_string(),
            AttributeValue::S("Verification successful".to_string()),
        )]);

        let attribute_map: AttributeMap = task_success.into();
        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_task_success_verify_from_attribute_map() {
        let attribute_map = HashMap::from([(
            "Verify".to_string(),
            AttributeValue::S("Verification successful".to_string()),
        )]);

        let expected_task_success = TaskSuccess::Verify {
            message: "Verification successful".to_string(),
        };

        let result: TaskSuccess = (&attribute_map).try_into().expect("Conversion failed");
        assert_eq!(result, expected_task_success);
    }

    #[test]
    fn test_task_result_failure_to_attribute_map() {
        let task_result = TaskResult::Failure(TaskFailure {
            error_type: ServerError::CompilationError,
            message: "Compilation failed".to_string(),
        });

        let expected_map = HashMap::from([(
            "Failure".to_string(),
            AttributeValue::Ss(vec![
                "CompilationError".into(),
                "Compilation failed".to_string(),
            ]),
        )]);

        let attribute_map: AttributeMap = task_result.into();
        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_task_result_failure_from_attribute_map() {
        let attribute_map = HashMap::from([(
            "Failure".to_string(),
            AttributeValue::Ss(vec![
                "CompilationError".into(),
                "Compilation failed".to_string(),
            ]),
        )]);

        let expected_task_result = TaskResult::Failure(TaskFailure {
            error_type: ServerError::CompilationError,
            message: "Compilation failed".to_string(),
        });

        let result: TaskResult = (&attribute_map).try_into().expect("Conversion failed");
        assert_eq!(result, expected_task_result);
    }

    #[test]
    fn test_task_result_success_compile_to_attribute_map() {
        let task_result = TaskResult::Success(TaskSuccess::Compile {
            presigned_urls: vec!["url1".to_string(), "url2".to_string()],
        });

        let expected_map = HashMap::from([(
            "Success".to_string(),
            AttributeValue::M(HashMap::from([(
                "Compile".to_string(),
                AttributeValue::Ss(vec!["url1".to_string(), "url2".to_string()]),
            )])),
        )]);

        let attribute_map: AttributeMap = task_result.into();
        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_task_result_success_compile_from_attribute_map() {
        let attribute_map = HashMap::from([(
            "Success".to_string(),
            AttributeValue::M(HashMap::from([(
                "Compile".to_string(),
                AttributeValue::Ss(vec!["url1".to_string(), "url2".to_string()]),
            )])),
        )]);

        let expected_task_result = TaskResult::Success(TaskSuccess::Compile {
            presigned_urls: vec!["url1".to_string(), "url2".to_string()],
        });

        let result: TaskResult = (&attribute_map).try_into().expect("Conversion failed");
        assert_eq!(result, expected_task_result);
    }

    #[test]
    fn test_task_result_success_verify_to_attribute_map() {
        let task_result = TaskResult::Success(TaskSuccess::Verify {
            message: "Verification successful".to_string(),
        });

        let expected_map = HashMap::from([(
            "Success".to_string(),
            AttributeValue::M(HashMap::from([(
                "Verify".to_string(),
                AttributeValue::S("Verification successful".to_string()),
            )])),
        )]);

        let attribute_map: AttributeMap = task_result.into();
        assert_eq!(attribute_map, expected_map);
    }

    #[test]
    fn test_task_result_success_verify_from_attribute_map() {
        let attribute_map = HashMap::from([(
            "Success".to_string(),
            AttributeValue::M(HashMap::from([(
                "Verify".to_string(),
                AttributeValue::S("Verification successful".to_string()),
            )])),
        )]);

        let expected_task_result = TaskResult::Success(TaskSuccess::Verify {
            message: "Verification successful".to_string(),
        });

        let result: TaskResult = (&attribute_map).try_into().expect("Conversion failed");
        assert_eq!(result, expected_task_result);
    }
}
