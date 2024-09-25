use crate::processor::compile_message_processor::CompileProcessor;
use types::item::TaskResult;

#[derive(thiserror::Error, Debug)]
pub enum UserProcessorError {}

pub trait UserDefinedProcessor: Send + Sync + 'static {
    type Message;
    fn process_message(&self, message: Self::Message) -> Result<TaskResult, UserProcessorError>;
}

// pub struct ZkSyncProcessor {
//     pub compile_processor: CompileProcessor<>
// }
