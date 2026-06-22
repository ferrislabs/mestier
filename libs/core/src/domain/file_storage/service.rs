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
        // Validate folder slug if provided before generating the key.
        if let Some(ref f) = command.folder {
            validate_folder_slug(f)?;
        }

        let key = self.generate_key(command.folder.as_deref());
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

    fn generate_key(&self, folder: Option<&str>) -> String {
        let id = generate_uuid_v7();
        match (self.key_prefix.is_empty(), folder) {
            (true, None) => id.to_string(),
            (true, Some(f)) => format!("{}/{}", f, id),
            (false, None) => format!("{}/{}", self.key_prefix, id),
            (false, Some(f)) => format!("{}/{}/{}", self.key_prefix, f, id),
        }
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

/// Validates that `folder` is a simple slug: lowercase letters, digits, hyphens, underscores.
/// Rejects empty strings, slashes, dots, and any other characters.
/// Pattern: ^[a-z0-9_-]+$
fn validate_folder_slug(folder: &str) -> Result<(), CoreError> {
    if folder.is_empty() {
        return Err(CoreError::Conflict(
            "invalid folder slug: must not be empty".to_owned(),
        ));
    }
    let valid = folder
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if !valid {
        return Err(CoreError::Conflict(format!(
            "invalid folder slug '{}': only [a-z0-9_-] allowed",
            folder
        )));
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
                folder: None,
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
                folder: None,
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

    #[tokio::test]
    async fn upload_with_folder_builds_key_under_subfolder() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage.clone(), "uploads");

        let file = service
            .upload(UploadFileCommand {
                mime_type: "image/png".to_owned(),
                bytes: vec![1, 2, 3],
                folder: Some("attachments".to_owned()),
            })
            .await
            .unwrap();

        // key must be: uploads/attachments/<uuid-v7>
        assert!(
            file.key.starts_with("uploads/attachments/"),
            "key was: {}",
            file.key
        );
    }

    #[tokio::test]
    async fn upload_without_folder_uses_flat_key() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage.clone(), "uploads");

        let file = service
            .upload(UploadFileCommand {
                mime_type: "text/plain".to_owned(),
                bytes: vec![42],
                folder: None,
            })
            .await
            .unwrap();

        // key must be: uploads/<uuid-v7>  (no sub-folder segment)
        let parts: Vec<&str> = file.key.split('/').collect();
        assert_eq!(parts.len(), 2, "key was: {}", file.key);
        assert_eq!(parts[0], "uploads");
    }

    #[tokio::test]
    async fn upload_rejects_folder_with_slash() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage.clone(), "uploads");

        let result = service
            .upload(UploadFileCommand {
                mime_type: "text/plain".to_owned(),
                bytes: vec![1],
                folder: Some("a/b".to_owned()),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn upload_rejects_folder_with_dotdot() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage.clone(), "uploads");

        let result = service
            .upload(UploadFileCommand {
                mime_type: "text/plain".to_owned(),
                bytes: vec![1],
                folder: Some("..".to_owned()),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn upload_rejects_empty_folder_string() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage.clone(), "uploads");

        let result = service
            .upload(UploadFileCommand {
                mime_type: "text/plain".to_owned(),
                bytes: vec![1],
                folder: Some("".to_owned()),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn upload_accepts_folder_with_hyphen_and_underscore() {
        let storage = FakeStorage::default();
        let service = FileStorageService::new(storage.clone(), "uploads");

        let file = service
            .upload(UploadFileCommand {
                mime_type: "image/jpeg".to_owned(),
                bytes: vec![0xff, 0xd8],
                folder: Some("chat-attachments_v2".to_owned()),
            })
            .await
            .unwrap();

        assert!(
            file.key.starts_with("uploads/chat-attachments_v2/"),
            "key was: {}",
            file.key
        );
    }
}
