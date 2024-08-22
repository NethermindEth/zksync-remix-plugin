use crate::types::Item;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;

#[derive(Clone)]
pub struct DynamoDBClient {
    client: Client,
    table_name: String,
}

impl DynamoDBClient {
    pub fn new(client: Client, table_name: &str) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }

    pub async fn delete_item(&self, id: String) {
        // TODO:
    }

    // TODO: remove unwraps
    pub async fn get_item(&self, id: String) -> Result<Option<Item>, ()> {
        let result = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key("ID", AttributeValue::S(id))
            .send()
            .await
            .unwrap();

        if let Some(item) = result.item {
            Ok(Some(item.try_into().unwrap()))
        } else {
            Ok(None)
        }
    }
}
