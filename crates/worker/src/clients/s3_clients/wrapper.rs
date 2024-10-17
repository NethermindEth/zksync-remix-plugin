use crate::clients::errors::{S3DeleteObjectError, S3Error, S3GetObjectError};
use crate::clients::retriable::{Retrier, State};
use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};
use aws_sdk_s3::Client;
use std::io::SeekFrom;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::{mpsc, oneshot};

use crate::clients::s3_clients::client::{S3Action, S3Client};
use crate::commands::CompilationFile;

#[derive(Clone)]
pub struct S3ClientWrapper {
    pub client: S3Client,
    actions_sender: mpsc::Sender<S3Action>,
    state: Arc<AtomicU8>,
}

impl S3ClientWrapper {
    pub fn new(client: Client, bucket_name: &str) -> Self {
        let client = S3Client::new(client, bucket_name);
        let state = Arc::new(AtomicU8::new(State::Connected as u8));
        let (sender, receiver) = mpsc::channel(1000);

        let retrier = Retrier::new(client.clone(), receiver, state.clone());
        tokio::spawn(retrier.start());

        Self {
            client,
            state,
            actions_sender: sender,
        }
    }

    pub async fn delete_dir(&self, dir: &str) -> Result<(), S3Error> {
        match self.state.load(Ordering::Acquire) {
            0 => match self.client.delete_dir_attempt(dir).await {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(S3Action::DeleteDir {
                dir: dir.to_string(),
                sender,
            })
            .await;
        receiver.await.unwrap()
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), S3DeleteObjectError> {
        match self.state.load(Ordering::Acquire) {
            0 => match self.client.delete_object_attempt(key).await {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(S3Action::DeleteObject {
                key: key.to_string(),
                sender,
            })
            .await;
        receiver.await.unwrap()
    }

    pub async fn extract_files(&self, dir: &str) -> Result<Vec<CompilationFile>, S3Error> {
        match self.state.load(Ordering::Acquire) {
            0 => match self.client.extract_files_attempt(dir).await {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(S3Action::ExtractFiles {
                dir: dir.to_string(),
                sender,
            })
            .await;
        receiver.await.unwrap()
    }

    pub async fn put_object(&self, key: &str, mut file: File) -> Result<(), S3Error> {
        match self.state.load(Ordering::Acquire) {
            0 => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).await?;
                match self.client.put_object_attempt(key, buf).await {
                    Ok(Some(val)) => return Ok(val),
                    Ok(None) => {
                        self.state
                            .store(State::Reconnecting as u8, Ordering::Release);
                        file.seek(SeekFrom::Start(0)).await?;
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(S3Action::PutObject {
                key: key.to_string(),
                file,
                sender,
            })
            .await;
        receiver.await.unwrap()
    }

    pub async fn get_object_presigned(
        &self,
        key: &str,
        expires_in: &PresigningConfig,
    ) -> Result<PresignedRequest, S3GetObjectError> {
        match self.state.load(Ordering::Acquire) {
            0 => match self
                .client
                .get_object_presigned_attempt(key, expires_in)
                .await
            {
                Ok(Some(val)) => return Ok(val),
                Ok(None) => self
                    .state
                    .store(State::Reconnecting as u8, Ordering::Release),
                Err(err) => return Err(err),
            },
            1 => {}
            _ => unreachable!(),
        }

        let (sender, receiver) = oneshot::channel();
        self.actions_sender
            .send(S3Action::GetObjectPresigned {
                key: key.to_string(),
                expires_in: expires_in.clone(),
                sender,
            })
            .await;
        receiver.await.unwrap()
    }
}
