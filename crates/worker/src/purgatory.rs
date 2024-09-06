use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinHandle};
use types::item::Status;
use types::SqsMessage;
use uuid::Uuid;

pub type Timestamp = u64;

#[derive(Clone)]
pub struct Purgatory {
    inner: Arc<Mutex<Inner>>,
}

impl Purgatory {
    pub fn new(state: State) -> Self {
        let mut handle = NonNull::dangling();
        let this = Self {
            inner: Arc::new(Mutex::new(Inner::new(handle, state))),
        };

        let initialized_handle = tokio::spawn(this.clone().deamon());
        unsafe {
            *handle.as_mut() = initialized_handle;
        }

        this
    }

    pub async fn purge(&mut self) {
        self.inner.lock().await.purge()
    }

    pub async fn add_task(&mut self, _: &SqsMessage) {
        todo!()
    }

    // TODO: args: status, id
    pub async fn update_task(&mut self) {}

    async fn deamon(self) {
        todo!()
    }
}

struct Inner {
    state: State,

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
    fn new(handle: NonNull<JoinHandle<()>>, state: State) -> Self {
        Self {
            handle,
            state,
            _marker: PhantomData,
        }
    }

    pub fn purge(&mut self) {
        todo!()
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
        todo!()
    }
}
