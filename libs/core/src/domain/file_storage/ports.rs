use common::CoreError;

use crate::domain::file_storage::{FileObject, PresignedUrl, StoredFile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileUpload {
    pub key: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

pub trait FileStorage: Clone + Send + Sync + 'static {
    fn upload(
        &self,
        upload: FileUpload,
    ) -> impl Future<Output = Result<StoredFile, CoreError>> + Send;

    fn get(&self, key: &str) -> impl Future<Output = Result<FileObject, CoreError>> + Send;

    /// A URL granting read access to `key` until it expires.
    ///
    /// `expires_in` is passed rather than decided here: how long a link should
    /// live is a policy the domain service owns, not the storage backend.
    fn presigned_get_url(
        &self,
        key: &str,
        expires_in: std::time::Duration,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn delete(&self, key: &str) -> impl Future<Output = Result<(), CoreError>> + Send;
}
