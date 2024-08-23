mod commands;
mod dynamodb_client;
mod errors;
mod sqs_client;
mod sqs_listener;
mod types;
mod utils;
mod worker;

use aws_config::BehaviorVersion;
use aws_runtime::env_config::file::{EnvConfigFileKind, EnvConfigFiles};
use std::ops::Deref;
use std::time::Duration;

use crate::{sqs_client::SqsClient, sqs_listener::SqsListener};

const AWS_PROFILE_DEFAULT: &str = "dev";
// TODO: remove
pub(crate) const QUEUE_URL_DEFAULT: &str =
    "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";

// TODO: state synchronization

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
    let sqs_client = SqsClient::new(sqs_client, QUEUE_URL_DEFAULT);

    let sqs_listener = SqsListener::new(sqs_client, Duration::from_secs(1));
    let sqs_receiver = sqs_listener.receiver();

    while let Ok(message) = sqs_receiver.recv().await {
        println!("{:?}", message);
        if let Some(receipt_handle) = message.receipt_handle {
            sqs_receiver
                .delete_message(receipt_handle)
                .await
                .map_err(|err| println!("delete error: {}", err.to_string()))
                .unwrap();
        }
    }
}
