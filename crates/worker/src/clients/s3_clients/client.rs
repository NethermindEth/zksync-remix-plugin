use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};
use aws_sdk_s3::types::Object;
use aws_sdk_s3::Client;
use aws_smithy_types::byte_stream::ByteStream;
use std::io::{SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::AtomicU8;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::oneshot;
use tracing::{error, warn};

use crate::clients::errors::{
    S3DeleteObjectError, S3Error, S3GetObjectError, S3ListObjectsError, S3PutObjectError,
};
use crate::clients::retriable::{handle_action_result, match_result, ActionHandler};
use crate::commands::CompilationFile;

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

    pub async fn extract_files_attempt(
        &self,
        dir: &str,
    ) -> Result<Option<Vec<CompilationFile>>, S3Error> {
        let result = self.extract_files(dir).await;
        match result {
            Err(S3Error::ListObjectsError(err)) => {
                match_result!(S3ListObjectsError, Err(err)).map_err(S3Error::from)
            }
            Err(S3Error::GetObjectError(err)) => {
                match_result!(S3GetObjectError, Err(err)).map_err(S3Error::from)
            }
            result => result.map(Some),
        }
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
        expires_in: &PresigningConfig,
    ) -> Result<PresignedRequest, S3GetObjectError> {
        self.client
            .get_object()
            .bucket(self.bucket_name.clone())
            .key(key.to_string())
            .presigned(expires_in.clone())
            .await
    }

    pub async fn get_object_presigned_attempt(
        &self,
        key: &str,
        expires_in: &PresigningConfig,
    ) -> Result<Option<PresignedRequest>, S3GetObjectError> {
        match_result!(
            S3GetObjectError,
            self.get_object_presigned(key, expires_in).await
        )
    }

    pub async fn put_object(
        &self,
        key: &str,
        data: impl Into<ByteStream>,
    ) -> Result<(), S3PutObjectError> {
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

    pub async fn put_object_attempt(
        &self,
        key: &str,
        data: impl Into<ByteStream>,
    ) -> Result<Option<()>, S3PutObjectError> {
        match_result!(S3PutObjectError, self.put_object(key, data).await)
    }

    pub async fn delete_dir(&self, dir: &str) -> Result<(), S3Error> {
        let objects = self.list_all_keys(dir).await?;
        // TODO: delete_objects instead
        for object in objects {
            let key = object.key().ok_or(S3Error::InvalidObjectError)?;
            self.delete_object(key).await?;
        }

        self.delete_object(dir).await?;
        Ok(())
    }

    pub async fn delete_dir_attempt(&self, dir: &str) -> Result<Option<()>, S3Error> {
        match self.delete_dir(dir).await {
            Err(S3Error::DeleteObjectError(err)) => {
                match_result!(S3DeleteObjectError, Err(err)).map_err(S3Error::from)
            }
            result => result.map(|value| Some(value)),
        }
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

    pub async fn delete_object_attempt(
        &self,
        key: &str,
    ) -> Result<Option<()>, S3DeleteObjectError> {
        match_result!(S3DeleteObjectError, self.delete_object(key).await)
    }

    pub async fn list_all_keys(&self, dir: &str) -> Result<Vec<Object>, S3ListObjectsError> {
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
    ExtractFiles {
        dir: String,
        sender: oneshot::Sender<Result<Vec<CompilationFile>, S3Error>>,
    },
    PutObject {
        key: String,
        file: File,
        sender: oneshot::Sender<Result<(), S3Error>>,
    },
    GetObjectPresigned {
        key: String,
        expires_in: PresigningConfig,
        sender: oneshot::Sender<Result<PresignedRequest, S3GetObjectError>>,
    },
}

impl ActionHandler for S3Client {
    type Action = S3Action;
    async fn handle(&self, action: Self::Action, state: Arc<AtomicU8>) -> Option<Self::Action> {
        match action {
            S3Action::Default => unreachable!(),
            S3Action::DeleteDir { dir, sender } => {
                let result = self.delete_dir_attempt(&dir).await;
                handle_action_result(result, sender, state)
                    .map(|sender| S3Action::DeleteDir { dir, sender })
            }
            S3Action::DeleteObject { key, sender } => {
                let result = self.delete_object_attempt(&key).await;
                handle_action_result(result, sender, state)
                    .map(|sender| S3Action::DeleteObject { key, sender })
            }
            S3Action::ExtractFiles { dir, sender } => {
                let result = self.extract_files_attempt(&dir).await;
                handle_action_result(result, sender, state)
                    .map(|sender| S3Action::ExtractFiles { dir, sender })
            }
            S3Action::GetObjectPresigned {
                key,
                expires_in,
                sender,
            } => {
                let result = self.get_object_presigned_attempt(&key, &expires_in).await;
                handle_action_result(result, sender, state).map(|sender| {
                    S3Action::GetObjectPresigned {
                        key,
                        expires_in,
                        sender,
                    }
                })
            }
            S3Action::PutObject {
                key,
                mut file,
                sender,
            } => {
                let mut buf = Vec::new();
                if let Err(err) = file.read_to_end(&mut buf).await {
                    let _ = sender.send(Err(S3Error::IoError(err)));
                    return None;
                };

                let result = self
                    .put_object_attempt(&key, buf)
                    .await
                    .map_err(S3Error::from);
                if let Some(sender) = handle_action_result(result, sender, state) {
                    if let Err(err) = file.seek(SeekFrom::Start(0)).await {
                        let _ = sender.send(Err(S3Error::IoError(err)));
                        None
                    } else {
                        Some(S3Action::PutObject { key, file, sender })
                    }
                } else {
                    None
                }
            }
        }
    }
}
