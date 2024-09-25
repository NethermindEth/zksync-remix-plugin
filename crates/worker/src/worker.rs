use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::error;

use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::processor::Processor;
use crate::sqs_listener::{SqsListener, SqsReceiver};

pub struct EngineBuilder {
    sqs_client: SqsClientWrapper,
    processor: Processor,
    running_workers: Vec<RunningEngine>,
}

impl EngineBuilder {
    pub fn new(sqs_client: SqsClientWrapper, processor: Processor) -> Self {
        EngineBuilder {
            sqs_client,
            processor,
            running_workers: vec![],
        }
    }

    pub fn start(self, num_workers: NonZeroUsize) -> RunningEngine {
        let sqs_listener = SqsListener::new(self.sqs_client, Duration::from_millis(500));

        RunningEngine::new(sqs_listener, self.processor, num_workers.get())
    }
}

pub struct RunningEngine {
    sqs_listener: SqsListener,
    num_workers: usize,
    worker_threads: Vec<JoinHandle<()>>,
}

impl RunningEngine {
    pub fn new(sqs_listener: SqsListener, processor: Processor, num_workers: usize) -> Self {
        let arc_processor = Arc::new(processor);
        let mut worker_threads = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            // Start worker
            let sqs_receiver = sqs_listener.receiver();
            let arc_processor_copy = arc_processor.clone();
            worker_threads.push(tokio::spawn(async move {
                RunningEngine::worker(sqs_receiver, arc_processor_copy).await;
            }));
        }

        Self {
            sqs_listener,
            num_workers,
            worker_threads,
        }
    }

    async fn worker(sqs_receiver: SqsReceiver, processor: Arc<Processor>) {
        while let Ok(message) = sqs_receiver.recv().await {
            let receipt_handle = if let Some(ref receipt_handle) = message.receipt_handle {
                receipt_handle.to_owned()
            } else {
                continue;
            };

            let sqs_message = match message.try_into() {
                Ok(val) => val,
                Err(err) => {
                    error!("Error converting into SqsMessage: {err}");
                    continue;
                }
            };

            // TODO: add metrics for how long it takes &
            // adjust "visibility timeout" or receiver chan capacity
            // TODO: add critical errors and return
            if let Err(err) = processor.process_message(sqs_message, receipt_handle).await {
                error!("{err}");
            }
        }
    }

    pub async fn wait(self) {
        futures::future::join_all(self.worker_threads).await;
    }
}
