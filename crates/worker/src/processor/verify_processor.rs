use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::processor::input_preparator::InputPreparator;
use crate::purgatory::Purgatory;

// TODO: make generic via adding MessageProcessor trait with process_message(...)
pub struct VerifyProcessor {
    sqs_client: SqsClientWrapper,
    s3_client: S3ClientWrapper,
    input_preparator: InputPreparator,
    purgatory: Purgatory,
}
