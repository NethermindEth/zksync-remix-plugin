use aws_sdk_dynamodb::types::AttributeValue;
use serde::Serialize;
use std::collections::HashMap;

pub const BUCKET_NAME_DEFAULT: &str = "zksync-compilation-s3";

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
        // // item_map.
        // match value.status {
        //     Status::Ready(val) => {
        //         item_map.insert("Data".into(), AttributeValue::S(val));
        //     }
        //     Status::Failed(val) => {
        //         item_map.insert("Data".into(), AttributeValue::S(val));
        //     }
        //     _ => {}
        // }

        item_map
    }
}
