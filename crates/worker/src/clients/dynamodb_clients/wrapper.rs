use aws_sdk_dynamodb::operation::scan::ScanOutput;
use aws_sdk_dynamodb::operation::update_item::builders::UpdateItemFluentBuilder;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;
use tokio::sync::mpsc;
use types::item::{Item, Status};

use crate::clients::dynamodb_clients::client::{DynamoDBAction, DynamoDBClient};
use crate::clients::errors::{DBDeleteError, DBError, DBScanError, DBUpdateError};
use crate::clients::retriable::{execute_retriable_operation, Retrier, State};

#[derive(Clone)]
pub struct DynamoDBClientWrapper {
    pub client: DynamoDBClient,
    actions_sender: mpsc::Sender<DynamoDBAction>,
    state: Arc<AtomicU8>,
}

impl DynamoDBClientWrapper {
    pub fn new(client: Client, table_name: &str) -> Self {
        let client = DynamoDBClient::new(client, table_name);
        let state = Arc::new(AtomicU8::new(State::Connected as u8));
        let (sender, receiver) = mpsc::channel(1000);

        let retrier = Retrier::new(client.clone(), receiver, state.clone());
        tokio::spawn(retrier.start());

        Self {
            client,
            state,
            actions_sender: sender,
        }
    }

    pub async fn delete_item(&self, id: &str) -> Result<(), DBDeleteError> {
        let operation = || self.client.delete_item_attempt(id);
        let action_factory = |sender| DynamoDBAction::DeleteItem {
            id: id.to_string(),
            sender,
        };

        // TODO: if all good. rewrite all other clients like that?
        execute_retriable_operation(operation, action_factory, &self.actions_sender, &self.state)
            .await
    }

    pub async fn get_item(&self, key: &str) -> Result<Option<Item>, DBError> {
        let operation = || self.client.get_item_attempt(key);
        let action_factory = |sender| DynamoDBAction::GetItem {
            id: key.to_string(),
            sender,
        };

        execute_retriable_operation(operation, action_factory, &self.actions_sender, &self.state)
            .await
    }

    pub async fn update_item_raw(
        &self,
        update_item_builder: &UpdateItemFluentBuilder,
    ) -> Result<(), DBUpdateError> {
        let operation = || self.client.update_item_raw_attempt(update_item_builder);

        let action_factory = |sender| DynamoDBAction::UpdateItemRaw {
            update_item_builder: update_item_builder.clone(),
            sender,
        };

        execute_retriable_operation(operation, action_factory, &self.actions_sender, &self.state)
            .await
    }

    pub async fn scan_items_prior_to(
        &self,
        time: &DateTime<Utc>,
        exclusive_start_key: &Option<HashMap<String, AttributeValue>>,
    ) -> Result<ScanOutput, DBScanError> {
        let operation = || {
            self.client
                .scan_items_prior_to_attempt(time, exclusive_start_key)
        };

        let action_factory = |sender| DynamoDBAction::ScanPriorTo {
            time: time.clone(),
            exclusive_start_key: exclusive_start_key.clone(),
            sender,
        };

        execute_retriable_operation(operation, action_factory, &self.actions_sender, &self.state)
            .await
    }

    pub async fn update_item_status_conditional(
        &self,
        id: &str,
        from: &Status,
        to: &Status,
    ) -> Result<(), DBUpdateError> {
        let operation = || {
            self.client
                .update_item_status_conditional_attempt(id, from, to)
        };

        let action_factory = |sender| DynamoDBAction::UpdateItemStatusConditional {
            id: id.to_string(),
            from: from.clone(),
            to: to.clone(),
            sender,
        };

        execute_retriable_operation(operation, action_factory, &self.actions_sender, &self.state)
            .await
    }
}
