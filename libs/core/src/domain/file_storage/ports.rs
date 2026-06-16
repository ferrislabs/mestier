use common::CoreError;

use crate::domain::file_storage::{FileObject, StoredFile};

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

    fn delete(&self, key: &str) -> impl Future<Output = Result<(), CoreError>> + Send;
}
