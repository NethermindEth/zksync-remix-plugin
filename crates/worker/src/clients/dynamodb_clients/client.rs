use aws_sdk_dynamodb::operation::scan::ScanOutput;
use aws_sdk_dynamodb::operation::update_item::builders::UpdateItemFluentBuilder;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;
use tokio::sync::oneshot;
use types::item::{Item, Status};

use crate::clients::errors::{DBDeleteError, DBError, DBGetError, DBScanError, DBUpdateError};
use crate::clients::retriable::{handle_action_result, match_result, ActionHandler};

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

    pub async fn delete_item(&self, id: &str) -> Result<(), DBDeleteError> {
        let _ = self
            .client
            .delete_item()
            .table_name(self.table_name.clone())
            .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn delete_item_attempt(&self, id: &str) -> Result<Option<()>, DBDeleteError> {
        match_result!(DBDeleteError, self.delete_item(id).await)
    }

    pub async fn update_item_raw(
        &self,
        update_item_builder: &UpdateItemFluentBuilder,
    ) -> Result<(), DBUpdateError> {
        let _ = update_item_builder.clone().send().await;
        Ok(())
    }

    pub async fn update_item_raw_attempt(
        &self,
        update_item_builder: &UpdateItemFluentBuilder,
    ) -> Result<Option<()>, DBUpdateError> {
        match_result!(
            DBUpdateError,
            self.update_item_raw(update_item_builder).await
        )
    }

    pub async fn update_item_status_conditional(
        &self,
        id: &str,
        from: &Status,
        to: &Status,
    ) -> Result<(), DBUpdateError> {
        let _ = self
            .client
            .update_item()
            .table_name(self.table_name.clone())
            .key(Item::primary_key_name(), AttributeValue::S(id.to_string()))
            .update_expression("SET #status = :toStatus")
            .condition_expression("#status = :fromStatus")
            .expression_attribute_names("#status", Status::attribute_name())
            .expression_attribute_values(":toStatus", AttributeValue::N(u32::from(to).to_string()))
            .expression_attribute_values(
                ":fromStatus",
                AttributeValue::N(u32::from(from).to_string()),
            )
            .send()
            .await?;
        Ok(())
    }

    pub async fn update_item_status_conditional_attempt(
        &self,
        id: &str,
        from: &Status,
        to: &Status,
    ) -> Result<Option<()>, DBUpdateError> {
        match_result!(
            DBUpdateError,
            self.update_item_status_conditional(id, from, to).await
        )
    }

    pub async fn get_item(&self, key: &str) -> Result<Option<Item>, DBError> {
        let result = self
            .client
            .get_item()
            .table_name(self.table_name.clone())
            .key(Item::primary_key_name(), AttributeValue::S(key.to_string()))
            .send()
            .await?;

        if let Some(item) = result.item {
            Ok(Some(item.try_into()?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_item_attempt(&self, key: &str) -> Result<Option<Option<Item>>, DBError> {
        match self.get_item(key).await {
            Err(DBError::GetItemError(err)) => {
                match_result!(DBGetError, Err(err)).map_err(DBError::from)
            }
            result => result.map(|value| Some(value)),
        }
    }

    pub async fn scan_items_prior_to(
        &self,
        time: &DateTime<Utc>,
        exclusive_start_key: &Option<HashMap<String, AttributeValue>>,
    ) -> Result<ScanOutput, DBScanError> {
        const MAX_CAPACITY: usize = 100;

        self.client
            .scan()
            .table_name(self.table_name.clone())
            .filter_expression("CreatedAt <= :created_at")
            .expression_attribute_values(":created_at", AttributeValue::S(time.to_rfc3339()))
            .limit(MAX_CAPACITY as i32)
            .set_exclusive_start_key(exclusive_start_key.clone())
            .send()
            .await
    }

    pub async fn scan_items_prior_to_attempt(
        &self,
        time: &DateTime<Utc>,
        exclusive_start_key: &Option<HashMap<String, AttributeValue>>,
    ) -> Result<Option<ScanOutput>, DBScanError> {
        match_result!(
            DBScanError,
            self.scan_items_prior_to(time, exclusive_start_key).await
        )
    }
}

#[derive(Default)]
pub enum DynamoDBAction {
    #[default]
    Default,
    DeleteItem {
        id: String,
        sender: oneshot::Sender<Result<(), DBDeleteError>>,
    },
    GetItem {
        id: String,
        sender: oneshot::Sender<Result<Option<Item>, DBError>>,
    },
    ScanPriorTo {
        time: DateTime<Utc>,
        exclusive_start_key: Option<HashMap<String, AttributeValue>>,
        sender: oneshot::Sender<Result<ScanOutput, DBScanError>>,
    },
    UpdateItemRaw {
        update_item_builder: UpdateItemFluentBuilder,
        sender: oneshot::Sender<Result<(), DBUpdateError>>,
    },
    UpdateItemStatusConditional {
        id: String,
        from: Status,
        to: Status,
        sender: oneshot::Sender<Result<(), DBUpdateError>>,
    },
}

impl ActionHandler for DynamoDBClient {
    type Action = DynamoDBAction;

    async fn handle(&self, action: Self::Action, state: Arc<AtomicU8>) -> Option<Self::Action> {
        match action {
            DynamoDBAction::Default => unreachable!(),
            DynamoDBAction::DeleteItem { id, sender } => {
                let result = self.delete_item_attempt(&id).await;
                handle_action_result(result, sender, state)
                    .map(|sender| DynamoDBAction::DeleteItem { id, sender })
            }
            DynamoDBAction::GetItem { id, sender } => {
                let result = self.get_item_attempt(&id).await;
                handle_action_result(result, sender, state)
                    .map(|sender| DynamoDBAction::GetItem { id, sender })
            }
            DynamoDBAction::ScanPriorTo {
                time,
                exclusive_start_key,
                sender,
            } => {
                let result = self
                    .scan_items_prior_to_attempt(&time, &exclusive_start_key)
                    .await;
                handle_action_result(result, sender, state).map(|sender| {
                    DynamoDBAction::ScanPriorTo {
                        time,
                        exclusive_start_key,
                        sender,
                    }
                })
            }
            DynamoDBAction::UpdateItemRaw {
                update_item_builder,
                sender,
            } => {
                let result = self.update_item_raw_attempt(&update_item_builder).await;
                handle_action_result(result, sender, state).map(|sender| {
                    DynamoDBAction::UpdateItemRaw {
                        update_item_builder,
                        sender,
                    }
                })
            }
            DynamoDBAction::UpdateItemStatusConditional {
                id,
                from,
                to,
                sender,
            } => {
                let result = self
                    .update_item_status_conditional_attempt(&id, &to, &from)
                    .await;
                handle_action_result(result, sender, state).map(|sender| {
                    DynamoDBAction::UpdateItemStatusConditional {
                        id,
                        to,
                        from,
                        sender,
                    }
                })
            }
        }
    }
}
