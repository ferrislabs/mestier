use mestier_core::StoredFile;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct FileUploadResponse {
    pub key: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

impl From<StoredFile> for FileUploadResponse {
    fn from(value: StoredFile) -> Self {
        Self {
            key: value.key,
            mime_type: value.mime_type,
            size_bytes: value.size_bytes,
        }
    }
}
