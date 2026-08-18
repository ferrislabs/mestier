use aws_credential_types::Credentials;
use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Builder as S3ConfigBuilder, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
};
use common::{CoreError, FileStorageConfig};

use crate::domain::file_storage::{
    FileObject, PresignedUrl, StoredFile,
    ports::{FileStorage, FileUpload},
};

#[derive(Clone)]
pub struct S3FileStorage {
    client: Client,
    bucket: String,
}

impl S3FileStorage {
    pub fn from_config(config: &FileStorageConfig) -> Self {
        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            None,
            None,
            "mestier",
        );
        let s3_config = S3ConfigBuilder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .endpoint_url(config.endpoint.clone())
            .credentials_provider(credentials)
            .force_path_style(config.force_path_style)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket: config.bucket.clone(),
        }
    }

    #[tracing::instrument(skip(self), fields(bucket = %self.bucket), err)]
    pub async fn ensure_bucket(&self) -> Result<(), CoreError> {
        match self.client.head_bucket().bucket(&self.bucket).send().await {
            Ok(_) => Ok(()),
            Err(err) if is_not_found_error(&err) => {
                self.client
                    .create_bucket()
                    .bucket(&self.bucket)
                    .send()
                    .await
                    .map_err(map_s3_error)?;

                Ok(())
            }
            Err(err) => Err(map_s3_error(err)),
        }
    }
}

impl FileStorage for S3FileStorage {
    #[tracing::instrument(skip(self, upload), fields(bucket = %self.bucket, key = %upload.key, mime_type = %upload.mime_type, size_bytes = upload.bytes.len()), err)]
    async fn upload(&self, upload: FileUpload) -> Result<StoredFile, CoreError> {
        let size_bytes = upload.bytes.len() as u64;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&upload.key)
            .content_type(upload.mime_type.clone())
            .content_length(size_bytes as i64)
            .body(ByteStream::from(upload.bytes))
            .send()
            .await
            .map_err(map_s3_error)?;

        Ok(StoredFile {
            key: upload.key,
            mime_type: upload.mime_type,
            size_bytes,
        })
    }

    #[tracing::instrument(skip(self), fields(bucket = %self.bucket, key = %key), err)]
    async fn get(&self, key: &str) -> Result<FileObject, CoreError> {
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(map_s3_error)?;
        let mime_type = object
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_owned();
        let content_length = object.content_length();
        let bytes = object
            .body
            .collect()
            .await
            .map_err(|e| CoreError::Internal(format!("failed to read s3 object body: {e}")))?
            .into_bytes()
            .to_vec();
        let size_bytes = content_length
            .and_then(|size| u64::try_from(size).ok())
            .unwrap_or(bytes.len() as u64);

        Ok(FileObject {
            key: key.to_owned(),
            mime_type,
            size_bytes,
            bytes,
        })
    }

    #[tracing::instrument(skip(self), fields(bucket = %self.bucket, key = %key), err)]
    async fn presigned_get_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> Result<PresignedUrl, CoreError> {
        // Read back from the config rather than adding `expires_in` to `now`:
        // the signature is what actually expires, and the two would drift if
        // the SDK ever clamped the duration.
        let config = PresigningConfig::expires_in(expires_in)
            .map_err(|e| CoreError::Internal(format!("invalid presigning config: {e}")))?;
        let expires_at = chrono::DateTime::<chrono::Utc>::from(config.start_time() + expires_in);

        let request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(config)
            .await
            .map_err(map_s3_error)?;

        Ok(PresignedUrl {
            url: request.uri().to_owned(),
            expires_at,
        })
    }

    #[tracing::instrument(skip(self), fields(bucket = %self.bucket, key = %key), err)]
    async fn delete(&self, key: &str) -> Result<(), CoreError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(map_s3_error)?;

        Ok(())
    }
}

fn map_s3_error<E>(err: E) -> CoreError
where
    E: std::fmt::Display,
{
    let message = err.to_string();
    if is_not_found_message(&message) {
        return CoreError::NotFound;
    }

    CoreError::Internal(format!("file storage error: {message}"))
}

fn is_not_found_error<E>(err: &E) -> bool
where
    E: std::fmt::Display,
{
    is_not_found_message(&err.to_string())
}

fn is_not_found_message(message: &str) -> bool {
    message.contains("NoSuchKey") || message.contains("NotFound") || message.contains("404")
}
