use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::warn;
use types::item::{Item, ItemError, Status, TaskResult};
use uuid::Uuid;

use crate::clients::dynamodb_clients::wrapper::DynamoDBClientWrapper;
use crate::clients::errors::DBError;
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::errors::PurgeError;
use crate::utils::lib::{s3_artifacts_dir, s3_compilation_files_dir, timestamp};

pub type Timestamp = u64;
pub const DURATION_TO_PURGE: u64 = 60 * 5; // 5 minutes
pub const PURGE_INTERVAL: Duration = Duration::from_secs(5 * 60);

#[derive(Clone)]
pub struct Purgatory {
    inner: Arc<Mutex<Inner>>,
}

impl Purgatory {
    pub fn new(db_client: DynamoDBClientWrapper, s3_client: S3ClientWrapper) -> Self {
        let handle = NonNull::dangling();
        let this = Self {
            inner: Arc::new(Mutex::new(Inner::new(
                handle,
                State::new(),
                db_client,
                s3_client,
            ))),
        };

        {
            let mut inner = this.inner.try_lock().unwrap();
            let mut initialized_handle = tokio::spawn(this.clone().daemon());
            inner.handle = unsafe { NonNull::new_unchecked(&mut initialized_handle as *mut _) };
        }

        this
    }

    pub async fn purge(&mut self) {
        self.inner.lock().await.purge().await;
    }

    pub async fn add_record(&mut self, id: Uuid, result: TaskResult) {
        self.inner.lock().await.add_record(id, result);
    }

    async fn daemon(mut self) {
        const PURGE_INTERVAL: Duration = Duration::from_secs(60);

        let mut interval = interval(PURGE_INTERVAL);
        loop {
            interval.tick().await;
            self.purge().await;
        }
    }
}
struct Inner {
    state: State,
    s3_client: S3ClientWrapper,
    db_client: DynamoDBClientWrapper,

    // No aliases possible since only we own the data
    handle: NonNull<JoinHandle<()>>,
    _marker: PhantomData<JoinHandle<()>>,
}

unsafe impl Send for Inner {}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            self.handle.as_ref().abort();
        }
    }
}

impl Inner {
    fn new(
        handle: NonNull<JoinHandle<()>>,
        state: State,
        db_client: DynamoDBClientWrapper,
        s3_client: S3ClientWrapper,
    ) -> Self {
        tokio::spawn(Self::global_state_purge(
            db_client.clone(),
            s3_client.clone(),
        ));

        Self {
            state,
            s3_client,
            db_client,
            handle,
            _marker: PhantomData,
        }
    }

    fn add_record(&mut self, id: Uuid, result: TaskResult) {
        let to_purge_timestampe = timestamp() + DURATION_TO_PURGE;

        self.state.task_status.insert(id, Status::Done(result));
        self.state
            .expiration_timestamps
            .push((id, to_purge_timestampe));
    }

    async fn global_state_purge(
        db_client: DynamoDBClientWrapper,
        s3_client: S3ClientWrapper,
    ) -> Result<(), PurgeError> {
        const SYNC_FROM_OFFSET: Option<Duration> = PURGE_INTERVAL.checked_mul(6);

        let mut global_state = GlobalState::new(db_client.clone());
        let sync_from = Utc::now() - SYNC_FROM_OFFSET.unwrap();

        loop {
            if global_state.sync(&sync_from).await.is_err() {
                break Ok(());
            }
            if global_state.items.is_empty() {
                break Ok(());
            }

            let items: Vec<Item> = global_state.items.drain(..).collect();
            for item in items {
                Inner::purge_item(&db_client, &s3_client, &item.id, &item.status).await?;
            }
        }
    }

    pub async fn purge_item(
        db_client: &DynamoDBClientWrapper,
        s3_client: &S3ClientWrapper,
        id: &Uuid,
        status: &Status,
    ) -> Result<(), PurgeError> {
        match status {
            Status::InProgress => warn!("Item compiling for too long!"),
            Status::Pending => {
                warn!("Item pending for too long");

                // Remove compilation files
                let files_dir = s3_compilation_files_dir(id.to_string().as_str());
                s3_client.delete_dir(&files_dir).await?;

                // Remove artifacts
                let artifacts_dir = s3_compilation_files_dir(id.to_string().as_str());
                s3_client.delete_dir(&artifacts_dir).await?;

                // Second would give neater replies
                db_client
                    .delete_item(id.to_string().as_str())
                    .await
                    .map_err(DBError::from)?;
            }
            Status::Done(_) => {
                let dir = s3_artifacts_dir(id.to_string().as_str());
                s3_client.delete_dir(&dir).await?;
                db_client
                    .delete_item(id.to_string().as_str())
                    .await
                    .map_err(DBError::from)?;
            }
        }

        Ok(())
    }

    pub async fn purge(&mut self) -> Result<(), PurgeError> {
        let now = timestamp();
        for (id, timestamp) in self.state.expiration_timestamps.iter() {
            if *timestamp > now {
                break;
            }

            let status = if let Some(status) = self.state.task_status.get(id) {
                status
            } else {
                warn!("Inconsistency! ID present vector but not in status map");
                continue;
            };

            Self::purge_item(&self.db_client, &self.s3_client, &id, &status).await?;
        }

        self.state.expiration_timestamps.retain(|(id, timestamp)| {
            if *timestamp > now {
                return true;
            };

            self.state.task_status.remove(id);
            false
        });

        Ok(())
    }
}

struct GlobalState {
    db_client: DynamoDBClientWrapper,
    pub items: Vec<Item>,
}

impl GlobalState {
    pub fn new(db_client: DynamoDBClientWrapper) -> Self {
        Self {
            db_client,
            items: vec![],
        }
    }

    pub async fn sync(&mut self, sync_from: &DateTime<Utc>) -> Result<(), PurgeError> {
        const MAX_CAPACITY: usize = 1000;

        let mut last_evaluated_key = None;
        loop {
            let output = self
                .db_client
                .scan_items_prior_to(sync_from, &last_evaluated_key)
                .await
                .map_err(DBError::from)?;
            let raw_items = if let Some(items) = output.items {
                items
            } else {
                break Ok(());
            };

            let remaining_capacity = MAX_CAPACITY - self.items.len();
            let mut to_append: Vec<Item> = raw_items
                .into_iter()
                .take(remaining_capacity)
                .map(|raw_item| raw_item.try_into())
                .collect::<Result<_, ItemError>>()?;

            self.items.append(&mut to_append);
            if self.items.len() == MAX_CAPACITY {
                break Ok(());
            }

            last_evaluated_key = if let Some(last_evaluated_key) = output.last_evaluated_key {
                Some(last_evaluated_key)
            } else {
                break Ok(());
            };
        }
    }
}

pub struct State {
    expiration_timestamps: Vec<(Uuid, Timestamp)>,
    task_status: HashMap<Uuid, Status>,
}

impl State {
    pub fn new() -> State {
        Self {
            expiration_timestamps: vec![],
            task_status: HashMap::new(),
        }
    }
}