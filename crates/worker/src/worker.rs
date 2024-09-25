use aws_sdk_sqs::types::Message;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::error;
use types::SqsMessage;

use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::processor::MessageProcessor;
use crate::processor::Processor;
use crate::sqs_listener::{SqsListener, SqsReceiver};

pub struct EngineBuilder<M: TryFrom<Message>, P: MessageProcessor<Message = M>> {
    sqs_client: SqsClientWrapper,
    processor: P,
    running_workers: Vec<RunningEngine>,
}

impl<M: TryFrom<Message>, P: MessageProcessor<Message = M>> EngineBuilder<M, P> {
    pub fn new(sqs_client: SqsClientWrapper, processor: P) -> Self {
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
    pub fn new<M: TryFrom<Message>, P: MessageProcessor<Message = M>>(
        sqs_listener: SqsListener,
        processor: P,
        num_workers: usize,
    ) -> Self {
        let processor = Arc::new(processor);
        let mut worker_threads = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            // Start worker
            let arc_processor = processor.clone();
            let sqs_receiver = sqs_listener.receiver();
            worker_threads.push(tokio::spawn(async move {
                RunningEngine::worker(sqs_receiver, arc_processor).await;
            }));
        }

        Self {
            sqs_listener,
            num_workers,
            worker_threads,
        }
    }

    async fn worker<M: TryFrom<Message>, P: MessageProcessor<Message = M>>(
        sqs_receiver: SqsReceiver,
        processor: Arc<P>,
    ) {
        while let Ok(message) = sqs_receiver.recv().await {
            let receipt_handle = if let Some(ref receipt_handle) = message.receipt_handle {
                receipt_handle.to_owned()
            } else {
                continue;
            };

            let sqs_message: M = match message.try_into() {
                Ok(val) => val,
                Err(err) => {
                    error!("Can't convert message to SqsMessage: {err}");
                    continue;
                }
            };

            // TODO: add metrics for how long it takes &
            // adjust "visibility timeout" or receiver chan capacity
            processor.process_message(sqs_message).await;
        }
    }

    pub async fn wait(self) {
        futures::future::join_all(self.worker_threads).await;
    }
}
