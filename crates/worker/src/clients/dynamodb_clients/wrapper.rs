use aws_sdk_dynamodb::operation::scan::ScanOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use aws_sdk_dynamodb::operation::update_item::builders::UpdateItemFluentBuilder;
use tokio::sync::{mpsc, oneshot};
use types::item::{Item, Status};

use crate::clients::dynamodb_clients::client::{DynamoDBAction, DynamoDBClient};
use crate::clients::errors::{DBDeleteError, DBError, DBScanError, DBUpdateError};
use crate::clients::retriable::{Retrier, State};

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
        match self.state.load(Ordering::Acquire) {
            0 => match self.client.delete_item_attempt(id).await {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err.into()),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(DynamoDBAction::DeleteItem {
                id: id.to_string(),
                sender,
            })
            .await;
        receiver.await.unwrap()
    }

    pub async fn get_item(&self, key: &str) -> Result<Option<Item>, DBError> {
        match self.state.load(Ordering::Acquire) {
            0 => match self.client.get_item_attempt(key).await {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err.into()),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(DynamoDBAction::GetItem {
                id: key.to_string(),
                sender,
            })
            .await;
        receiver.await.unwrap()
    }

    pub async fn update_item_raw(&self, update_item_builder: UpdateItemFluentBuilder) -> Result<(), DBUpdateError> {
       match self.state.load(Ordering::Acquire) {
            0 => match self
                .client
                .update_item_raw_attempt(update_item_builder.clone())
                .await
            {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err.into()),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(DynamoDBAction::UpdateItemRaw {
                update_item_builder,
                sender
            })
            .await;
        receiver.await.unwrap()
    }

    pub async fn scan_items_prior_to(
        &self,
        time: &DateTime<Utc>,
        exclusive_start_key: &Option<HashMap<String, AttributeValue>>,
    ) -> Result<ScanOutput, DBScanError> {
        match self.state.load(Ordering::Acquire) {
            0 => match self
                .client
                .scan_items_prior_to_attempt(time, exclusive_start_key)
                .await
            {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err.into()),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(DynamoDBAction::ScanPriorTo {
                time: time.clone(),
                exclusive_start_key: exclusive_start_key.clone(),
                sender,
            })
            .await;
        receiver.await.unwrap()
    }

    pub async fn update_item_status_conditional(
        &self,
        id: &str,
        from: &Status,
        to: &Status,
    ) -> Result<(), DBUpdateError> {
        match self.state.load(Ordering::Acquire) {
            0 => match self
                .client
                .update_item_status_conditional_attempt(id, from, to)
                .await
            {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err.into()),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(DynamoDBAction::UpdateItemStatusConditional {
                id: id.to_string(),
                from: from.clone(),
                to: to.clone(),
                sender,
            })
            .await;
        receiver.await.unwrap()
    }
}
