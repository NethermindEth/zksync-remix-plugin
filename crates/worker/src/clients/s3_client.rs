use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};
use aws_sdk_s3::types::Object;
use aws_sdk_s3::Client;
use aws_smithy_types::byte_stream::ByteStream;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::{error, warn};

use crate::clients::errors::{S3DeleteObjectError, S3Error};
use crate::clients::retriable::{handle_action_result, match_result, ActionHandler, State};
use crate::commands::compile::CompilationFile;

#[derive(Clone)]
pub struct S3Client {
    pub client: Client,
    pub bucket_name: String,
}

impl S3Client {
    pub fn new(client: Client, bucket_name: &str) -> Self {
        Self {
            bucket_name: bucket_name.to_string(),
            client,
        }
    }

    pub async fn extract_files(&self, dir: &str) -> Result<Vec<CompilationFile>, S3Error> {
        let objects = self.list_all_keys(dir).await?;

        let mut files = vec![];
        for object in objects {
            let key = object.key().ok_or(S3Error::InvalidObjectError)?;
            let expected_size = object.size.ok_or(S3Error::InvalidObjectError)?;

            let mut contents = Vec::with_capacity(expected_size as usize);
            self.get_object_into(key, &mut contents).await?;
            if contents.len() as i64 != expected_size {
                error!("Fetched num bytes != expected size of file.");
                return Err(S3Error::InvalidObjectError);
            }

            let file_path = Path::new(key)
                .strip_prefix(dir)
                .expect("Unreachable. list_all_keys bug.");
            files.push(CompilationFile {
                file_content: contents,
                file_path: file_path.to_path_buf(),
            });
        }

        Ok(files)
    }

    pub async fn get_object_into(&self, key: &str, writer: &mut impl Write) -> Result<(), S3Error> {
        let mut object = self
            .client
            .get_object()
            .bucket(self.bucket_name.clone())
            .key(key)
            .send()
            .await?;

        while let Some(bytes) = object.body.try_next().await? {
            writer.write_all(&bytes)?;
        }

        Ok(())
    }

    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, S3Error> {
        let mut contents = vec![];
        self.get_object_into(key, &mut contents).await?;

        Ok(contents)
    }

    pub async fn get_object_presigned(
        &self,
        key: &str,
        expires_in: PresigningConfig,
    ) -> Result<PresignedRequest, S3Error> {
        Ok(self
            .client
            .get_object()
            .bucket(self.bucket_name.clone())
            .key(key.to_string())
            .presigned(expires_in)
            .await
            .map_err(S3Error::from)?)
    }

    pub async fn put_object(&self, key: &str, data: impl Into<ByteStream>) -> Result<(), S3Error> {
        let _ = self
            .client
            .put_object()
            .bucket(self.bucket_name.clone())
            .key(key.to_string())
            .body(data.into())
            .send()
            .await?;

        Ok(())
    }

    pub async fn delete_dir(&self, dir: &str) -> Result<(), S3Error> {
        let objects = self.list_all_keys(dir).await?;
        for object in objects {
            let key = object.key().ok_or(S3Error::InvalidObjectError)?;
            self.delete_object(key).await?;
        }

        self.delete_object(dir).await?;
        Ok(())
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), S3DeleteObjectError> {
        let _ = self
            .client
            .delete_object()
            .bucket(self.bucket_name.clone())
            .key(key)
            .send()
            .await?;

        Ok(())
    }

    pub async fn list_all_keys(&self, dir: &str) -> Result<Vec<Object>, S3Error> {
        let mut objects = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(self.bucket_name.clone())
                .delimiter('/')
                .prefix(dir.to_string());
            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await?;
            if let Some(contents) = response.contents {
                objects.extend(contents);
            }

            let is_truncated = if let Some(is_truncated) = response.is_truncated {
                is_truncated
            } else {
                warn!("is_truncated empty");
                break;
            };

            if !is_truncated {
                break;
            }

            continuation_token = response.next_continuation_token;
            if continuation_token.is_none() {
                error!("continuation_token wasn't set!");
                break;
            }
        }

        Ok(objects)
    }
}

#[derive(Default)]
pub enum S3Action {
    #[default]
    Default,
    DeleteDir {
        dir: String,
        sender: oneshot::Sender<Result<(), S3Error>>,
    },
    DeleteObject {
        key: String,
        sender: oneshot::Sender<Result<(), S3DeleteObjectError>>,
    },
}

impl ActionHandler for S3Client {
    type Action = S3Action;
    async fn handle(&self, action: Self::Action, state: Arc<AtomicU8>) -> Option<Self::Action> {
        match action {
            S3Action::Default => unreachable!(),
            S3Action::DeleteDir { dir, sender } => {
                let result = self.delete_dir(&dir).await;
                match result {
                    Ok(()) => {
                        state.store(State::Connected as u8, Ordering::Release);
                        let _ = sender.send(Ok(()));
                        None
                    }
                    Err(S3Error::DeleteObjectError(err)) => {
                        let result = match_result!(S3DeleteObjectError, Err(err));
                        let result = result.map_err(S3Error::from);
                        handle_action_result(result, sender, state)
                            .map(|sender| S3Action::DeleteDir { dir, sender })
                    }
                    Err(err) => {
                        let _ = sender.send(Err(err));
                        None
                    }
                }
            }
            S3Action::DeleteObject { key, sender } => {
                let result = self.delete_object(&key).await;
                let result = match_result!(S3DeleteObjectError, result);
                handle_action_result(result, sender, state)
                    .map(|sender| S3Action::DeleteObject { key, sender })
            }
        }
    }
}
