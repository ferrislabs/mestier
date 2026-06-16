use serde::Serialize;
use utoipa::ToSchema;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct StoredFile {
    pub key: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileObject {
    pub key: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub bytes: Vec<u8>,
}

impl FileObject {
    pub fn metadata(&self) -> StoredFile {
        StoredFile {
            key: self.key.clone(),
            mime_type: self.mime_type.clone(),
            size_bytes: self.size_bytes,
        }
    }
}
