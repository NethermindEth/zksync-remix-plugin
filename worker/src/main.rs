use aws_config::profile::profile_file::ProfileFiles;
use aws_config::BehaviorVersion;
use aws_runtime::env_config::file::{EnvConfigFileKind, EnvConfigFiles};

const AWS_PROFILE_DEFAULT: &str = "dev";
const QUEUE_URL_DEFAULT: &str = "https://sqs.ap-southeast-2.amazonaws.com/266735844848/zksync-sqs";



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

    // Example: Send a message to an SQS queue
    let send_result = sqs_client
        .send_message()
        .queue_url(QUEUE_URL_DEFAULT)
        .message_body("Hello from Rust!")
        .send()
        .await
        .map_err(|err| println!("{}", err.to_string()))
        .expect("Oops");

    let receive = sqs

    println!("{:?}", send_result);
}
