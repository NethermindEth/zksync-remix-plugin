use crate::errors::DBError;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use types::item::Item;

#[derive(Clone)]
pub struct DynamoDBClient {
    pub client: Client,
    pub table_name: String,
}

impl DynamoDBClient {
    pub fn new(client: Client, table_name: &str) -> Self {
        Self {
            client,
            table_name: table_name.into(),
        }
    }

    pub async fn delete_item(&self, id: String) -> Result<(), DBError> {
        self.client
            .delete_item()
            .table_name(self.table_name.clone())
            .key(Item::primary_key_name(), AttributeValue::S(id))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_item(&self, id: String) -> Result<Option<Item>, DBError> {
        let result = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key(Item::primary_key_name(), AttributeValue::S(id))
            .send()
            .await?;

        if let Some(item) = result.item {
            // TODO: maybe change status or delete when error?
            Ok(Some(item.try_into()?))
        } else {
            Ok(None)
        }
    }
}
