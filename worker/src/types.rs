// TODO: move to separate crate

use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::macos::raw::stat;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SqsMessage {
    Compile { id: Uuid },
    Verify { id: Uuid },
}

#[derive(Debug, Clone, Serialize)]
pub enum Status {
    // TODO: add FilesUploaded(?)
    Pending,
    Compiling,
    Ready(String),
    Failed(String),
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
    // TODO: error
    type Error = ();
    fn try_from(value: &HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let status = value.get("Status").ok_or(())?;
        let status: u32 = status.as_n().map_err(|_| ())?.parse::<u32>().map_err(|_| ())?;
        let status = match status {
            0 => Status::Pending,
            1 => Status::Compiling,
            2 => {
                let data = value.get("Data").ok_or(())?;
                let data = data.as_s().map_err(|_|())?;

                Status::Ready(data.clone())
            }
            3 => {
                let data = value.get("Data").ok_or(())?;
                let data = data.as_s().map_err(|_|())?;

                Status::Failed(data.clone())
            }
            _ => return Err(()),
        };

        Ok(status)
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for Item {
    // TODO: error
    type Error = ();
    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Item, Self::Error> {
        let id = value.get("ID").ok_or(())?;
        let id = id.as_s().map_err(|_| ())?;
        let status = (&value).try_into()?;

        Ok(Item {
            id: id.clone(),
            status,
        })
    }
}
