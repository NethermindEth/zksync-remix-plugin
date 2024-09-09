use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::warn;
use types::item::Status;
use types::{SqsMessage, ARTIFACTS_FOLDER};
use uuid::Uuid;

use crate::dynamodb_client::DynamoDBClient;
use crate::s3_client::S3Client;
use crate::utils::lib::timestamp;

pub type Timestamp = u64;

#[derive(Clone)]
pub struct Purgatory {
    inner: Arc<Mutex<Inner>>,
}

impl Purgatory {
    pub fn new(state: State, db_client: DynamoDBClient, s3_client: S3Client) -> Self {
        let mut handle = NonNull::dangling();
        let this = Self {
            inner: Arc::new(Mutex::new(Inner::new(handle, state, db_client, s3_client))),
        };

        let initialized_handle = tokio::spawn(this.clone().deamon());
        unsafe {
            *handle.as_mut() = initialized_handle;
        }

        this
    }

    pub async fn purge(&mut self) {
        self.inner.lock().await.purge().await;
    }

    pub async fn add_task(&mut self, _: &SqsMessage) {
        todo!()
    }

    // TODO: args: status, id
    pub async fn update_task(&mut self) {}

    async fn deamon(self) {
        const PURGE_INTERVAL: Duration = Duration::from_secs(60);

        loop {
            let mut inner = self.inner.lock().await;
            inner.purge().await;

            sleep(PURGE_INTERVAL).await;
        }
    }
}

struct Inner {
    state: State,
    s3_client: S3Client,
    db_client: DynamoDBClient,

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
        db_client: DynamoDBClient,
        s3_client: S3Client,
    ) -> Self {
        Self {
            state,
            s3_client,
            db_client,
            handle,
            _marker: PhantomData,
        }
    }

    pub async fn purge(&mut self) {
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

            match status {
                Status::Pending => warn!("Item pending for too long!"),
                Status::Compiling => warn!("Item compiling for too long!"),
                Status::Ready { .. } => {
                    let dir = format!("{}/{}/", ARTIFACTS_FOLDER, id);
                    self.s3_client.delete_dir(&dir).await.unwrap(); // TODO: fix
                    self.db_client
                        .delete_item(id.to_string().as_str())
                        .await
                        .unwrap();
                }
                Status::Failed { .. } => {
                    let dir = format!("{}/{}/", ARTIFACTS_FOLDER, id);
                    self.s3_client.delete_dir(&dir).await; // TODO: fix
                    self.db_client
                        .delete_item(id.to_string().as_str())
                        .await
                        .unwrap();
                }
            }
        }

        self.state
            .expiration_timestamps
            .retain(|(_, timestamp)| *timestamp > now);
    }

    // TODO: replace with Self::purge
    // pub async fn supervisor(
    //     db_client: DynamoDBClient,
    //     expiration_timestamps: Arc<Mutex<Vec<(Uuid, Timestamp)>>>,
    // ) {
    //     loop {
    //         let now = timestamp();
    //
    //         let to_delete = {
    //             let mut to_delete = vec![];
    //             let mut expiration_timestamps = expiration_timestamps.lock().await;
    //             expiration_timestamps.retain(|&(uuid, expiration)| {
    //                 if expiration < now {
    //                     to_delete.push(uuid);
    //                     false
    //                 } else {
    //                     true
    //                 }
    //             });
    //
    //             to_delete
    //         };
    //
    //         for uuid in to_delete {
    //             db_client.delete_item(uuid.to_string()).await;
    //         }
    //
    //         sleep(Duration::from_millis(2000)).await;
    //     }
    // }
}

pub struct State {
    expiration_timestamps: Vec<(Uuid, Timestamp)>,
    task_status: HashMap<Uuid, Status>,
}

impl State {
    pub async fn load() -> State {
        // TODO: load state here from DB
        Self {
            expiration_timestamps: vec![],
            task_status: HashMap::new(),
        }
    }
}
