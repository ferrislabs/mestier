use aws_credential_types::Credentials;
use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Builder as S3ConfigBuilder, Region},
    error::ProvideErrorMetadata,
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

// `SdkError<E, R>::to_string()` is deliberately opaque — for the
// `ServiceError` variant it prints the literal string "service error",
// never the inner error's code or message (see aws-smithy-runtime-api's
// `Display for SdkError`). String-matching that output for "NotFound" /
// "NoSuchKey" never actually worked: a real 404 from `head_bucket` (which
// has no response body to embed a code in) rendered as "service error" and
// fell straight into the `Internal` branch, so `ensure_bucket` could never
// take its "create the bucket" path and instead crashed the caller that
// unwraps `create_service` — a real incident, not a hypothetical.
// `ProvideErrorMetadata::code()` reads the AWS error code straight from the
// parsed response metadata, bypassing that broken hop entirely: `SdkError`
// delegates `.meta()` to the inner error for the `ServiceError` variant, so
// this works uniformly across every error type this module deals with.
fn map_s3_error<E>(err: E) -> CoreError
where
    E: ProvideErrorMetadata + std::fmt::Debug,
{
    if is_not_found_code(err.code()) {
        return CoreError::NotFound;
    }

    let detail = match (err.code(), err.message()) {
        (Some(code), Some(message)) => format!("{code}: {message}"),
        (Some(code), None) => code.to_owned(),
        // No code at all means this isn't a modeled service error (e.g. a
        // dispatch failure or timeout) — Debug is verbose but it's the only
        // source of detail left at that point.
        (None, _) => format!("{err:?}"),
    };

    CoreError::Internal(format!("file storage error: {detail}"))
}

fn is_not_found_error<E>(err: &E) -> bool
where
    E: ProvideErrorMetadata,
{
    is_not_found_code(err.code())
}

fn is_not_found_code(code: Option<&str>) -> bool {
    matches!(
        code,
        Some("NoSuchKey") | Some("NoSuchBucket") | Some("NotFound")
    )
}

#[cfg(test)]
mod tests {
    use aws_sdk_s3::{
        error::{ErrorMetadata, SdkError},
        operation::head_bucket::HeadBucketError,
        types::error::NotFound,
    };

    use super::*;

    #[test]
    fn is_not_found_code_matches_the_known_s3_codes() {
        assert!(is_not_found_code(Some("NoSuchKey")));
        assert!(is_not_found_code(Some("NoSuchBucket")));
        assert!(is_not_found_code(Some("NotFound")));
        assert!(!is_not_found_code(Some("InternalError")));
        assert!(!is_not_found_code(None));
    }

    // Regression test for the incident this module caused: `head_bucket` on a
    // missing bucket has no response body, so the SDK can't attach a code to
    // `SdkError`'s own `Display` (it always prints "service error", see the
    // comment above `map_s3_error`) — only `ProvideErrorMetadata::code()`,
    // read through the delegation `SdkError` does to the inner service
    // error, actually carries "NotFound" through.
    #[test]
    fn a_head_bucket_not_found_service_error_is_recognized_as_not_found() {
        // Mirrors what the SDK's own response deserializer does (see
        // `de_head_bucket_http_error` in aws-sdk-s3): the parsed HTTP-status-
        // derived metadata is attached to the `NotFound` payload via
        // `.meta(generic)`, not left as the builder's empty default.
        let meta = ErrorMetadata::builder().code("NotFound").build();
        let err: SdkError<HeadBucketError, ()> = SdkError::service_error(
            HeadBucketError::NotFound(NotFound::builder().meta(meta).build()),
            (),
        );

        assert!(is_not_found_error(&err));
        assert!(matches!(map_s3_error(err), CoreError::NotFound));
    }

    #[test]
    fn a_different_service_error_is_not_mistaken_for_not_found() {
        let meta = ErrorMetadata::builder()
            .code("InternalError")
            .message("something else broke")
            .build();
        let err: SdkError<HeadBucketError, ()> =
            SdkError::service_error(HeadBucketError::generic(meta), ());

        assert!(!is_not_found_error(&err));
        assert!(
            matches!(map_s3_error(err), CoreError::Internal(msg) if msg.contains("InternalError"))
        );
    }
}
