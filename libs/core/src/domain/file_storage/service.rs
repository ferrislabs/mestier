use common::{CoreError, generate_uuid_v7};

use crate::domain::file_storage::{
    FileObject, StoredFile,
    commands::UploadFileCommand,
    ports::{FileStorage, FileUpload},
};

#[derive(Clone)]
pub struct FileStorageService<S>
where
    S: FileStorage,
{
    storage: S,
    key_prefix: String,
}

impl<S> FileStorageService<S>
where
    S: FileStorage,
{
    pub fn new(storage: S, key_prefix: impl Into<String>) -> Self {
        Self {
            storage,
            key_prefix: normalize_prefix(key_prefix.into()),
        }
    }

    #[tracing::instrument(skip(self, command), fields(mime_type = %command.mime_type, size_bytes = command.bytes.len()), err)]
    pub async fn upload(&self, command: UploadFileCommand) -> Result<StoredFile, CoreError> {
        let key = self.generate_key();
        let upload = FileUpload {
            key,
            mime_type: normalize_mime_type(command.mime_type),
            bytes: command.bytes,
        };

        self.storage.upload(upload).await
    }

    #[tracing::instrument(skip(self), fields(key = %key), err)]
    pub async fn get(&self, key: &str) -> Result<FileObject, CoreError> {
        validate_key(key)?;
        self.storage.get(key).await
    }

    #[tracing::instrument(skip(self), fields(key = %key), err)]
    pub async fn delete(&self, key: &str) -> Result<(), CoreError> {
        validate_key(key)?;
        self.storage.delete(key).await
    }

    fn generate_key(&self) -> String {
        let id = generate_uuid_v7();
        if self.key_prefix.is_empty() {
            return id.to_string();
        }

        format!("{}/{}", self.key_prefix, id)
    }
}

fn normalize_prefix(prefix: String) -> String {
    prefix
        .trim()
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_mime_type(mime_type: String) -> String {
    let trimmed = mime_type.trim();
    if trimmed.is_empty() {
        return "application/octet-stream".to_owned();
    }

    trimmed.to_owned()
}

fn validate_key(key: &str) -> Result<(), CoreError> {
    let invalid = key.is_empty()
        || key.starts_with('/')
        || key.contains("..")
        || key.chars().any(char::is_control);

    if invalid {
        return Err(CoreError::Conflict("invalid file key".to_owned()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct FakeStorage {
        uploads: Arc<Mutex<Vec<FileUpload>>>,
    }

    impl FileStorage for FakeStorage {
        async fn upload(&self, upload: FileUpload) -> Result<StoredFile, CoreError> {
            let stored = StoredFile {
                key: upload.key.clone(),
                mime_type: upload.mime_type.clone(),
                size_bytes: upload.bytes.len() as u64,
            };
            self.uploads.lock().unwrap().push(upload);
            Ok(stored)
        }

        async fn get(&self, key: &str) -> Result<FileObject, CoreError> {
            Ok(FileObject {
                key: key.to_owned(),
                mime_type: "text/plain".to_owned(),
                size_bytes: 4,
                bytes: b"test".to_vec(),
            })
        }

        async fn delete(&self, _key: &str) -> Result<(), CoreError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn upload_generates_key_under_prefix() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage.clone(), "/uploads//photos/");

        let file = service
            .upload(UploadFileCommand {
                mime_type: "image/png".to_owned(),
                bytes: vec![1, 2, 3],
            })
            .await
            .unwrap();

        assert!(file.key.starts_with("uploads/photos/"));
        assert_eq!(file.mime_type, "image/png");
        assert_eq!(file.size_bytes, 3);
    }

    #[tokio::test]
    async fn upload_defaults_blank_mime_type() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage.clone(), "");

        let file = service
            .upload(UploadFileCommand {
                mime_type: " ".to_owned(),
                bytes: vec![1],
            })
            .await
            .unwrap();

        assert_eq!(file.mime_type, "application/octet-stream");
    }

    #[tokio::test]
    async fn get_rejects_invalid_keys() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage, "");

        assert!(service.get("../secret").await.is_err());
        assert!(service.get("/secret").await.is_err());
        assert!(service.get("").await.is_err());
    }
}
