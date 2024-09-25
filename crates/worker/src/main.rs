mod clients;
mod commands;
mod compile_processor;
mod errors;
mod input_preparator;
mod processor;
mod purgatory;
mod sqs_listener;
mod utils;
mod worker;

use aws_config::BehaviorVersion;
use aws_runtime::env_config::file::{EnvConfigFileKind, EnvConfigFiles};
use std::num::NonZeroUsize;

use crate::clients::dynamodb_clients::wrapper::DynamoDBClientWrapper;
use crate::clients::s3_clients::wrapper::S3ClientWrapper;
use crate::clients::sqs_clients::wrapper::SqsClientWrapper;
use crate::compile_processor::CompileProcessor;
use crate::processor::Processor;
use crate::purgatory::Purgatory;
use crate::worker::EngineBuilder;

const AWS_PROFILE_DEFAULT: &str = "dev";

// TODO: remove all of the below. Impl cli.
pub(crate) const QUEUE_URL_DEFAULT: &str =
    "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";
const TABLE_NAME_DEFAULT: &str = "zksync-table";
const BUCKET_NAME_DEFAULT: &str = "zksync-compilation-s3";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .without_time() // CloudWatch will add the ingestion time
        .with_target(false)
        .init();

    let profile_name = std::env::var("AWS_PROFILE").unwrap_or(AWS_PROFILE_DEFAULT.into());
    let profile_files = EnvConfigFiles::builder()
        .with_file(EnvConfigFileKind::Credentials, "./credentials")
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .profile_files(profile_files)
        .profile_name(profile_name)
        .region("ap-southeast-2")
        .load()
        .await;

    // Initialize SQS client
    let sqs_client = aws_sdk_sqs::Client::new(&config);
    let sqs_client = SqsClientWrapper::new(sqs_client, QUEUE_URL_DEFAULT);

    // Initialize DynamoDb client
    let db_client = aws_sdk_dynamodb::Client::new(&config);
    let db_client = DynamoDBClientWrapper::new(db_client, TABLE_NAME_DEFAULT);

    // Initialize S3 client
    let s3_client = aws_sdk_s3::Client::new(&config);
    let s3_client = S3ClientWrapper::new(s3_client, BUCKET_NAME_DEFAULT);

    let purgatory = Purgatory::new(db_client.clone(), s3_client.clone());

    // Initialize processors
    let compile_processor = CompileProcessor::new(sqs_client.clone(), s3_client.clone(), purgatory.clone());
    let processor = Processor:: new(db_client, s3_client, sqs_client.clone(), compile_processor, purgatory);

    // Engine
    let engine = EngineBuilder::new(sqs_client, processor);
    let running_engine = engine.start(NonZeroUsize::new(10).unwrap());

    running_engine.wait().await;

    // TODO: transfer metrics.
}
