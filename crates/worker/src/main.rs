mod commands;
mod dynamodb_client;
mod errors;
mod s3_client;
mod sqs_client;
mod sqs_listener;
mod utils;
mod worker;

use aws_config::BehaviorVersion;
use aws_runtime::env_config::file::{EnvConfigFileKind, EnvConfigFiles};
use std::num::NonZeroUsize;

use crate::dynamodb_client::DynamoDBClient;
use crate::s3_client::S3Client;
use crate::sqs_client::wrapper::SqsClientWrapper;
use crate::sqs_client::SqsClient;
use crate::worker::EngineBuilder;

const AWS_PROFILE_DEFAULT: &str = "dev";
// TODO: remove
pub(crate) const QUEUE_URL_DEFAULT: &str =
    "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";
const TABLE_NAME_DEFAULT: &str = "zksync-table";
const BUCKET_NAME_DEFAULT: &str = "zksync-compilation-s3";

// TODO: state synchronization for purging

#[tokio::main]
async fn main() {
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
    let db_client = DynamoDBClient::new(db_client, TABLE_NAME_DEFAULT);

    // Initialize S3 client
    let s3_client = aws_sdk_s3::Client::new(&config);
    let s3_client = S3Client::new(s3_client, BUCKET_NAME_DEFAULT);

    let mut engine = EngineBuilder::new(sqs_client, db_client, s3_client, true);
    let running_engine = engine.start(NonZeroUsize::new(10).unwrap());

    running_engine.wait().await;
}
